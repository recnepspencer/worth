//! Logical query identity; neither source custody nor permission to reconstruct a read.
use std::{cmp::Ordering, sync::Arc};
use worth_execution::ExecutionMemoryReservation;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::identity::{EntityId, KindId};

#[derive(Clone, Debug)]
pub struct StructuralWalkKey(pub(super) Arc<Parameters>);

#[derive(Debug)]
pub struct Parameters {
    pub relation: Arc<str>,
    pub entity_kind: KindId,
    pub relation_kind: KindId,
    pub direction: RelationDirection,
    pub roots: Vec<EntityId>,
    pub _memory: Option<ExecutionMemoryReservation>,
}

impl StructuralWalkKey {
    pub fn relation(&self) -> &str {
        &self.0.relation
    }
    pub fn roots(&self) -> &[EntityId] {
        &self.0.roots
    }
    pub fn entity_kind(&self) -> KindId {
        self.0.entity_kind
    }
    pub fn relation_kind(&self) -> KindId {
        self.0.relation_kind
    }
    pub fn direction(&self) -> RelationDirection {
        self.0.direction
    }
    pub fn write_locator(
        &self,
        output: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        write!(
            output,
            "application-structural-walk:{}#{}:{}:{}:{:?}:roots#{}",
            self.relation().len(),
            self.relation(),
            self.entity_kind().as_u32(),
            self.relation_kind().as_u32(),
            self.direction(),
            self.roots().len()
        )?;
        for id in self.roots() {
            write!(
                output,
                ":{}:{}:{}",
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value()
            )?;
        }
        Ok(())
    }
    fn identity(&self) -> (&str, KindId, KindId, bool, &[EntityId]) {
        (
            self.relation(),
            self.entity_kind(),
            self.relation_kind(),
            self.direction() == RelationDirection::Reverse,
            self.roots(),
        )
    }
}
impl PartialEq for StructuralWalkKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.identity() == other.identity()
    }
}
impl Eq for StructuralWalkKey {}
impl Ord for StructuralWalkKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if Arc::ptr_eq(&self.0, &other.0) {
            Ordering::Equal
        } else {
            self.identity().cmp(&other.identity())
        }
    }
}
impl PartialOrd for StructuralWalkKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
#[path = "key_ordering_tests.rs"]
mod key_ordering_tests;

mod court_doors_for_test;
