use serde::{Deserialize, Serialize};
use worth_foundational::facade::{AspectKey, CanonicalFieldPath};

use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::DerivedIndexId;
use crate::storage::data::{AuthoritativeFieldComparisonKey, RelationalFieldPresence};
use crate::transactions::data::RecordRef;

/// Relational's sealed description of what one canonical commit changed.
/// An old encoding with no touch graph cannot assert an exact empty change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalDescriptiveTouchGraph {
    Exact(Vec<RelationalDescriptiveTouch>),
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalDescriptiveTouchPrecision {
    Exact,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelationalTouchAdjacencyDirection {
    Outgoing,
    Incoming,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelationalDescriptiveTouch {
    EntityLifecycle {
        entity: EntityId,
        kind: KindId,
    },
    RelationLifecycle {
        relation: RelationId,
        kind: KindId,
    },
    AspectRevision {
        record: RecordRef,
        aspect: AspectKey,
    },
    FieldRevision {
        record: RecordRef,
        kind: KindId,
        aspect: AspectKey,
        path: CanonicalFieldPath,
        presence: RelationalFieldPresence,
    },
    RelationMembership {
        relation: RelationId,
        kind: KindId,
        source: EntityId,
        target: EntityId,
    },
    AdjacencyRevision {
        kind: KindId,
        anchor: EntityId,
        direction: RelationalTouchAdjacencyDirection,
    },
    IndexMembership {
        index: DerivedIndexId,
        kind: KindId,
        aspect: AspectKey,
        path: CanonicalFieldPath,
        old_key: Option<AuthoritativeFieldComparisonKey>,
        new_key: Option<AuthoritativeFieldComparisonKey>,
    },
    IndexDefinition {
        index: DerivedIndexId,
    },
}

impl RelationalDescriptiveTouchGraph {
    pub(crate) fn exact(mut touches: Vec<RelationalDescriptiveTouch>) -> Self {
        touches.sort();
        touches.dedup();
        Self::Exact(touches)
    }

    pub(crate) const fn unavailable() -> Self {
        Self::Unavailable
    }

    pub const fn precision(&self) -> RelationalDescriptiveTouchPrecision {
        match self {
            Self::Exact(_) => RelationalDescriptiveTouchPrecision::Exact,
            Self::Unavailable => RelationalDescriptiveTouchPrecision::Unavailable,
        }
    }

    pub fn exact_touches(&self) -> Option<&[RelationalDescriptiveTouch]> {
        match self {
            Self::Exact(touches) => Some(touches),
            Self::Unavailable => None,
        }
    }
}

impl super::CanonicalCommitEnvelope {
    /// Late Native consequences carry the same actual adjacency effects as ordinary
    /// mutation. Full generations and kinds come from the mutation owner, never
    /// from dense patch detail or invented authored endpoint intents.
    pub(crate) fn complete_native_adjacency(
        &mut self,
        deltas: &[crate::authority::mutation::AdjacencyDelta],
    ) {
        if deltas.is_empty() {
            return;
        }
        let RelationalDescriptiveTouchGraph::Exact(touches) = &mut self.descriptive_touches else {
            return;
        };
        for delta in deltas {
            let (source, target) = match delta.kind {
                crate::authority::mutation::AdjacencyDeltaKind::Created { source, target }
                | crate::authority::mutation::AdjacencyDeltaKind::Deleted { source, target } => {
                    (source, target)
                }
            };
            touches.push(RelationalDescriptiveTouch::RelationLifecycle {
                relation: delta.relation_id,
                kind: delta.kind_id,
            });
            touches.push(RelationalDescriptiveTouch::RelationMembership {
                relation: delta.relation_id,
                kind: delta.kind_id,
                source,
                target,
            });
            for (anchor, direction) in [
                (source, RelationalTouchAdjacencyDirection::Outgoing),
                (target, RelationalTouchAdjacencyDirection::Incoming),
            ] {
                touches.push(RelationalDescriptiveTouch::AdjacencyRevision {
                    kind: delta.kind_id,
                    anchor,
                    direction,
                });
            }
        }
        touches.sort();
        touches.dedup();
    }
}
