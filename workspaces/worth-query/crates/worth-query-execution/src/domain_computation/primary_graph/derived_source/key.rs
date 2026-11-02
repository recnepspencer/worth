//! Logical read identity is descriptive; the observation separately retains its issuer.
use super::*;
use std::cmp::Ordering;
use worth_foundational::facade::RelationDirection;
use worth_relational::facade::{identity::EntityId, schema::DerivedRelationIdentity};
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct DerivedSourceKey(
    pub(super) Arc<Parameters>,
);
#[derive(Debug)]
pub(super) struct Parameters {
    pub(super) relation: String,
    pub(super) identity: DerivedRelationIdentity,
    pub(super) root: EntityId,
    pub(super) direction: RelationDirection,
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}
impl DerivedSourceKey {
    pub(in crate::domain_computation::primary_graph) fn relation(&self) -> &str {
        &self.0.relation
    }
    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &DerivedRelationIdentity {
        &self.0.identity
    }
    pub(in crate::domain_computation::primary_graph) fn root(&self) -> EntityId {
        self.0.root
    }
    pub(in crate::domain_computation::primary_graph) fn direction(&self) -> RelationDirection {
        self.0.direction
    }
    pub(in crate::domain_computation::primary_graph) fn write_locator(
        &self,
        output: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        let id = self.identity();
        write!(
            output,
            "application-derived-source:{}#{}:{}#{}:{}:{}#{}:{}:{}:{}:{:?}",
            self.relation().len(),
            self.relation(),
            id.schema.0.len(),
            id.schema.0,
            id.version.0,
            id.name.len(),
            id.name,
            self.root().partition_value(),
            self.root().local_slot_value(),
            self.root().generation_value(),
            self.direction()
        )
    }
    pub(super) fn same_with_poll<E>(
        &self,
        other: &Self,
        poll: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        if self.root() != other.root()
            || self.direction() != other.direction()
            || self.identity().version != other.identity().version
        {
            return Ok(false);
        }
        for (a, b) in [
            (self.relation(), other.relation()),
            (
                self.identity().schema.0.as_str(),
                other.identity().schema.0.as_str(),
            ),
            (
                self.identity().name.as_str(),
                other.identity().name.as_str(),
            ),
        ] {
            if a.len() != b.len() {
                poll()?;
                return Ok(false);
            }
            for (a, b) in a.bytes().zip(b.bytes()) {
                poll()?;
                if a != b {
                    return Ok(false);
                }
            }
        }
        poll()?;
        Ok(true)
    }
    fn ordered(&self) -> (&str, &DerivedRelationIdentity, EntityId, bool) {
        (
            self.relation(),
            self.identity(),
            self.root(),
            self.direction() == RelationDirection::Reverse,
        )
    }
}
impl PartialEq for DerivedSourceKey {
    fn eq(&self, other: &Self) -> bool {
        self.ordered() == other.ordered()
    }
}
impl Eq for DerivedSourceKey {}
impl Ord for DerivedSourceKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ordered().cmp(&other.ordered())
    }
}
impl PartialOrd for DerivedSourceKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
