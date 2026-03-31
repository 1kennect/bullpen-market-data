//! The profile graph on the store.

use serde_json::json;

use super::{Fake, SCOPE, caller, client};
use crate::core::traits::memory::profile::ProfileGraph;
use crate::core::types::memory::profile::{ProfileEntity, ProfileFact, ProfileRelation};
use crate::stores::http::HttpProfileGraph;

fn takes() -> ProfileRelation {
    ProfileRelation {
        subject: ProfileEntity {
            kind: "person".into(),
            label: "me".into(),
        },
        relation: "holds".into(),
        object: ProfileEntity {
            kind: "security".into(),
            label: "AAPL".into(),
        },
        confidence: 0.8,
    }
}

#[tokio::test]
async fn facts_go_up_whole_and_nothing_is_sent_for_none() {
    let fake = Fake::default();
    fake.on(
        "POST",
        &format!("{SCOPE}/profile/facts"),
        200,
        json!({"stored": 1}),
    );
    let graph = HttpProfileGraph::new(client(&fake).await);
    assert!(graph.upsert(&caller(), &[]).await.is_ok());
    assert!(fake.seen().is_empty());

    let relation = takes();
    let fact = ProfileFact {
        subject: relation.subject.clone(),
        relation: relation.relation.clone(),
        object: relation.object.clone(),
        confidence: 0.8,
    };
    assert!(graph.upsert(&caller(), &[fact]).await.is_ok());
    let body = fake.last().body;
    assert_eq!(body["facts"][0]["subject"]["label"], "me");
    assert_eq!(body["facts"][0]["relation"], "holds");
    assert_eq!(body["facts"][0]["object"]["kind"], "security");
}

#[tokio::test]
async fn nodes_and_relations_come_from_the_profile_route() {
    let fake = Fake::default();
    fake.on(
        "GET",
        &format!("{SCOPE}/profile"),
        200,
        json!({
            "nodes": [{
                "id": "0d3c9a6e-2b7f-4a8e-8f61-2c1f5e9b7a10",
                "kind": "security",
                "label": "AAPL",
                "confidence": 0.8,
                "created_at": "2026-10-08T01:00:00",
                "updated_at": "2026-10-08T02:00:00"
            }],
            "relations": [{
                "subject": {"kind": "person", "label": "me"},
                "relation": "holds",
                "object": {"kind": "security", "label": "AAPL"},
                "confidence": 0.8
            }]
        }),
    );
    let graph = HttpProfileGraph::new(client(&fake).await);
    let Ok(nodes) = graph.recall(&caller(), 7).await else {
        unreachable!("nodes load")
    };
    assert_eq!(
        fake.last().query.get("limit").map(String::as_str),
        Some("7")
    );
    assert_eq!(nodes[0].label, "AAPL");
    let Ok(relations) = graph.relations(&caller(), 7).await else {
        unreachable!("relations load")
    };
    assert_eq!(relations, vec![takes()]);
}

#[tokio::test]
async fn matching_dropping_and_forgetting_use_their_routes() {
    let fake = Fake::default();
    fake.on(
        "GET",
        &format!("{SCOPE}/profile/matching"),
        200,
        json!({"relations": []}),
    );
    fake.on(
        "DELETE",
        &format!("{SCOPE}/profile/relations"),
        200,
        json!({"deleted": true}),
    );
    fake.on(
        "DELETE",
        &format!("{SCOPE}/profile/nodes"),
        200,
        json!({"deleted": 1}),
    );
    fake.on(
        "DELETE",
        &format!("{SCOPE}/data"),
        200,
        json!({"deleted": 4}),
    );
    let graph = HttpProfileGraph::new(client(&fake).await);

    assert!(matches!(graph.matching(&caller(), "me", "holds").await, Ok(r) if r.is_empty()));
    let asked = fake.last();
    assert_eq!(asked.query.get("subject").map(String::as_str), Some("me"));
    assert_eq!(
        asked.query.get("relation").map(String::as_str),
        Some("holds")
    );

    assert!(matches!(
        graph.drop_relation(&caller(), &takes()).await,
        Ok(true)
    ));
    assert_eq!(
        fake.last().body,
        json!({"subject": "me", "relation": "holds", "object": "AAPL"})
    );

    assert!(matches!(graph.forget(&caller(), "AAPL").await, Ok(1)));
    assert_eq!(
        fake.last().query.get("label").map(String::as_str),
        Some("AAPL")
    );
    assert!(matches!(graph.forget_all(&caller()).await, Ok(4)));
}
