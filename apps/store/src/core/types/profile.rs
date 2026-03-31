//! Profile graph wire shapes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full confidence, the value a fact that says nothing about it carries.
fn certain() -> f32 {
    1.0
}

/// One entity, by kind and label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// What sort of thing it is.
    pub kind: String,
    /// How it is named.
    pub label: String,
}

/// One fact to write: two nodes and the edge between them.
#[derive(Debug, Deserialize)]
pub struct Fact {
    /// Entity the relation starts at.
    pub subject: Entity,
    /// What the relation is.
    pub relation: String,
    /// Entity the relation points to.
    pub object: Entity,
    /// How sure extraction is, 0 to 1.
    #[serde(default = "certain")]
    pub confidence: f32,
}

/// POST /profile/facts.
#[derive(Debug, Deserialize)]
pub struct FactsBody {
    /// Facts to write.
    pub facts: Vec<Fact>,
}

/// One stored node.
#[derive(Debug, Serialize)]
pub struct Node {
    /// Row id.
    pub id: Uuid,
    /// What sort of thing it is.
    pub kind: String,
    /// How it is named.
    pub label: String,
    /// Highest confidence any fact gave it, 0 to 1.
    pub confidence: f32,
    /// When it first entered the graph.
    pub created_at: DateTime<Utc>,
    /// When a fact last confirmed it.
    pub updated_at: DateTime<Utc>,
}

/// One stored relation with both ends resolved.
#[derive(Debug, Serialize)]
#[allow(clippy::struct_field_names)]
pub struct Relation {
    /// Where the relation starts.
    pub subject: Entity,
    /// What it asserts.
    pub relation: String,
    /// Where it ends.
    pub object: Entity,
    /// Confidence of the latest fact that wrote it.
    pub confidence: f32,
}

/// GET /profile/matching.
#[derive(Debug, Deserialize)]
pub struct MatchingQuery {
    /// Subject label, compared without case.
    pub subject: String,
    /// Relation, compared exactly.
    pub relation: String,
}

/// DELETE /profile/relations: one relation by its labels.
#[derive(Debug, Deserialize)]
pub struct RelationKey {
    /// Subject label, compared without case.
    pub subject: String,
    /// Relation, compared exactly.
    pub relation: String,
    /// Object label, compared without case.
    pub object: String,
}

/// DELETE /profile/nodes.
#[derive(Debug, Deserialize)]
pub struct LabelQuery {
    /// Label of the nodes to remove, compared without case.
    pub label: String,
}
