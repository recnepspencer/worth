//! Complete structural reads from one sealed immutable Native root.
use worth_foundational::facade::RelationDirection;

use crate::identity::data::{EntityId, KindId};

pub mod complete;
pub mod memory;
pub mod outcome;
pub mod traversal;

pub use outcome::{
    BoundedPinnedStructuralWalk, PinnedStructuralWalk, PinnedStructuralWalkBudget,
    PinnedStructuralWalkDenial, PinnedStructuralWalkDenialKind, PinnedStructuralWalkLimit,
    PinnedStructuralWalkReads, PinnedStructuralWalkVisit,
};

/// Read constraints only; the pinned receiver and original request grant access.
pub struct PinnedStructuralWalkScope<'input> {
    roots: &'input [EntityId],
    entity_kind: KindId,
    relation_kind: KindId,
    direction: RelationDirection,
    maximum_work: Option<usize>,
    budget: Option<PinnedStructuralWalkBudget>,
    control:
        Option<&'input dyn Fn() -> Result<(), crate::execution::RelationalExecutionDenialCause>>,
    #[cfg(test)]
    observer: Option<&'input dyn Fn(PinnedWalkEventForTest)>,
}

impl<'input> PinnedStructuralWalkScope<'input> {
    pub fn new(
        roots: &'input [EntityId],
        entity_kind: KindId,
        relation_kind: KindId,
        direction: RelationDirection,
    ) -> Self {
        Self {
            roots,
            entity_kind,
            relation_kind,
            direction,
            maximum_work: None,
            budget: None,
            control: None,
            #[cfg(test)]
            observer: None,
        }
    }

    /// An optional caller read limit, independent of observations and request work.
    pub fn with_work_limit(mut self, maximum: Option<usize>) -> Self {
        self.maximum_work = maximum;
        self
    }

    /// Borrow an additional caller refusal checkpoint. It can only narrow the
    /// original request's controls; it supplies no execution or success authority.
    pub fn with_control(
        mut self,
        control: &'input dyn Fn() -> Result<(), crate::execution::RelationalExecutionDenialCause>,
    ) -> Self {
        self.control = Some(control);
        self
    }

    #[cfg(test)]
    pub(crate) fn observe_for_test(
        mut self,
        observer: &'input dyn Fn(PinnedWalkEventForTest),
    ) -> Self {
        self.observer = Some(observer);
        self
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub enum PinnedWalkEventForTest {
    Entity(EntityId),
    Relation(crate::identity::data::RelationId),
    Adjacency(EntityId),
}
