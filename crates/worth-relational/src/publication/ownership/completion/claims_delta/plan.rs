//! Ordered desired-state reconciliation retains physical custody on equality.
use crate::publication::ownership::record::RelationClaim;
use crate::{
    identity::data::{EntityId, RelationId},
    transactions::data::RecordRef,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Claim {
    Member(EntityId),
    Relation(RelationClaim),
}
#[derive(Debug, Clone)]
pub(super) enum StoredClaim {
    Member {
        entity: EntityId,
        link: RelationId,
    },
    Relation {
        edge: RelationClaim,
        witness: EntityId,
        link: RelationId,
    },
}
#[derive(Debug)]
pub(super) enum Change {
    Retire(usize),
    Insert(usize),
    Revise { previous: usize, next: usize },
}
impl Claim {
    pub(super) fn key(&self) -> RecordRef {
        match self {
            Self::Member(id) => RecordRef::Entity(*id),
            Self::Relation(edge) => RecordRef::Relation(edge.id),
        }
    }
}
impl StoredClaim {
    pub(super) fn key(&self) -> RecordRef {
        match self {
            Self::Member { entity, .. } => RecordRef::Entity(*entity),
            Self::Relation { edge, .. } => RecordRef::Relation(edge.id),
        }
    }
    pub(super) fn link(&self) -> RelationId {
        match self {
            Self::Member { link, .. } | Self::Relation { link, .. } => *link,
        }
    }
    fn equals(&self, claim: &Claim) -> bool {
        match (self, claim) {
            (Self::Member { entity, .. }, Claim::Member(next)) => entity == next,
            (Self::Relation { edge, .. }, Claim::Relation(next)) => edge == next,
            _ => false,
        }
    }
}
/// Inputs are admitted, unique and sorted by record identity. Cost: old + desired;
/// output storage is bounded by their sum, and only actual changes are applied.
pub(super) fn changes(previous: &[StoredClaim], desired: &[Claim]) -> Vec<Change> {
    let mut changes = Vec::with_capacity(previous.len() + desired.len());
    let (mut old, mut next) = (0, 0);
    while old < previous.len() || next < desired.len() {
        let ordering = match (previous.get(old), desired.get(next)) {
            (Some(old), Some(next)) => old.key().cmp(&next.key()),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => break,
        };
        match ordering {
            std::cmp::Ordering::Less => {
                changes.push(Change::Retire(old));
                old += 1;
            }
            std::cmp::Ordering::Greater => {
                changes.push(Change::Insert(next));
                next += 1;
            }
            std::cmp::Ordering::Equal => {
                if !previous[old].equals(&desired[next]) {
                    changes.push(Change::Revise {
                        previous: old,
                        next,
                    });
                }
                old += 1;
                next += 1;
            }
        }
    }
    changes
}
#[cfg(test)]
mod tests;
