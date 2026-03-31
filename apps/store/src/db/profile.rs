//! The profile graph: nodes keyed by kind and label without case, and the relations between them.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Db;
use crate::core::types::Scope;
use crate::core::types::error::Error;
use crate::core::types::profile::{Entity, Fact, Node, Relation, RelationKey};

/// One node row.
type NodeRow = (Uuid, String, String, f32, DateTime<Utc>, DateTime<Utc>);

/// One relation row: subject kind and label, relation, object kind and label, confidence.
type RelationRow = (String, String, String, String, String, f32);

/// Selects relations of scope with both ends; callers add filters and order.
const RELATIONS: &str = "select s.kind, s.label, r.relation, o.kind, o.label, r.confidence
     from profile_relations r
     join profile_nodes s on s.id = r.subject_id
     join profile_nodes o on o.id = r.object_id
     where s.tenant_id = $1 and s.user_id = $2";

impl Db {
    /// Writes each fact as two nodes and an edge, raising what is already there.
    pub async fn upsert(&self, scope: &Scope, facts: &[Fact]) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        for fact in facts {
            let subject = node(&mut tx, scope, &fact.subject, fact.confidence).await?;
            let object = node(&mut tx, scope, &fact.object, fact.confidence).await?;
            sqlx::query(
                "insert into profile_relations (subject_id, relation, object_id, confidence)
                 values ($1, $2, $3, $4)
                 on conflict (subject_id, relation, object_id)
                 do update set confidence = excluded.confidence, updated_at = now()",
            )
            .bind(subject)
            .bind(&fact.relation)
            .bind(object)
            .bind(fact.confidence)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Nodes of scope, most confident and most recently confirmed first.
    pub async fn nodes(&self, scope: &Scope, limit: usize) -> Result<Vec<Node>, Error> {
        let rows: Vec<NodeRow> = sqlx::query_as(
            "select id, kind, label, confidence, created_at, updated_at from profile_nodes
             where tenant_id = $1 and user_id = $2
             order by confidence desc, updated_at desc
             limit $3",
        )
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(bound(limit))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(id, kind, label, confidence, created_at, updated_at)| Node {
                    id,
                    kind,
                    label,
                    confidence,
                    created_at,
                    updated_at,
                },
            )
            .collect())
    }

    /// Relations of scope, most confident first.
    pub async fn relations(&self, scope: &Scope, limit: usize) -> Result<Vec<Relation>, Error> {
        let rows: Vec<RelationRow> = sqlx::query_as(&format!(
            "{RELATIONS} order by r.confidence desc, r.updated_at desc limit $3"
        ))
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(bound(limit))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(relation).collect())
    }

    /// Relations of scope from subject under relation, up to limit.
    pub async fn matching(
        &self,
        scope: &Scope,
        subject: &str,
        relation_name: &str,
        limit: usize,
    ) -> Result<Vec<Relation>, Error> {
        let rows: Vec<RelationRow> = sqlx::query_as(&format!(
            "{RELATIONS} and lower(s.label) = lower($3) and r.relation = $4
             order by r.confidence desc, r.updated_at desc limit $5"
        ))
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(subject)
        .bind(relation_name)
        .bind(bound(limit))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(relation).collect())
    }

    /// Removes one relation of scope by its labels. Returns whether it was there.
    pub async fn drop_relation(&self, scope: &Scope, key: &RelationKey) -> Result<bool, Error> {
        let done = sqlx::query(
            "delete from profile_relations r
             using profile_nodes s, profile_nodes o
             where r.subject_id = s.id and r.object_id = o.id
               and s.tenant_id = $1 and s.user_id = $2
               and lower(s.label) = lower($3) and r.relation = $4 and lower(o.label) = lower($5)",
        )
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(&key.subject)
        .bind(&key.relation)
        .bind(&key.object)
        .execute(&self.pool)
        .await?;
        Ok(done.rows_affected() > 0)
    }

    /// Removes the nodes of scope under label and every relation through them; returns the count.
    pub async fn forget(&self, scope: &Scope, label: &str) -> Result<u64, Error> {
        let done = sqlx::query(
            "delete from profile_nodes
             where tenant_id = $1 and user_id = $2 and lower(label) = lower($3)",
        )
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(label)
        .execute(&self.pool)
        .await?;
        Ok(done.rows_affected())
    }

    /// Removes every node and memory of scope; returns the rows removed.
    pub async fn forget_all(&self, scope: &Scope) -> Result<u64, Error> {
        let mut tx = self.pool.begin().await?;
        let mut removed = 0;
        for table in ["profile_nodes", "memories"] {
            removed += sqlx::query(&format!(
                "delete from {table} where tenant_id = $1 and user_id = $2"
            ))
            .bind(&scope.tenant)
            .bind(&scope.user)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        }
        tx.commit().await?;
        Ok(removed)
    }
}

/// Upserts one node of scope, keeping the higher confidence; returns its id.
async fn node(
    tx: &mut sqlx::PgConnection,
    scope: &Scope,
    entity: &Entity,
    confidence: f32,
) -> Result<Uuid, Error> {
    let (id,): (Uuid,) = sqlx::query_as(
        "insert into profile_nodes (id, tenant_id, user_id, kind, label, confidence)
         values ($1, $2, $3, $4, $5, $6)
         on conflict (tenant_id, user_id, kind, lower(label))
         do update set confidence = greatest(profile_nodes.confidence, excluded.confidence),
                       updated_at = now()
         returning id",
    )
    .bind(Uuid::now_v7())
    .bind(&scope.tenant)
    .bind(&scope.user)
    .bind(&entity.kind)
    .bind(&entity.label)
    .bind(confidence)
    .fetch_one(&mut *tx)
    .await?;
    Ok(id)
}

/// A relation from its row.
fn relation(
    (subject_kind, subject_label, relation, object_kind, object_label, confidence): RelationRow,
) -> Relation {
    Relation {
        subject: Entity {
            kind: subject_kind,
            label: subject_label,
        },
        relation,
        object: Entity {
            kind: object_kind,
            label: object_label,
        },
        confidence,
    }
}

/// A limit as a bind value.
fn bound(limit: usize) -> i64 {
    i64::try_from(limit).unwrap_or(i64::MAX)
}
