//! Complete structural reads from one sealed immutable Native root.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::RelationDirection;

use super::super::RelationalPinnedRead;
use crate::identity::data::{EntityId, KindId};
use crate::visibility::materialization::read_records::projection::VisibilityProjectionView;

mod memory;
mod outcome;
mod traversal;

pub use outcome::{
    PinnedStructuralWalk, PinnedStructuralWalkDenial, PinnedStructuralWalkDenialKind,
    PinnedStructuralWalkReads, PinnedStructuralWalkVisit,
};

/// Read constraints only; the pinned receiver and original request grant access.
pub struct PinnedStructuralWalkScope<'input> {
    roots: &'input [EntityId],
    entity_kind: KindId,
    relation_kind: KindId,
    direction: RelationDirection,
    maximum_work: Option<usize>,
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
            #[cfg(test)]
            observer: None,
        }
    }

    /// An optional caller read limit, independent of observations and request work.
    pub fn with_work_limit(mut self, maximum: Option<usize>) -> Self {
        self.maximum_work = maximum;
        self
    }

    #[cfg(test)]
    fn observe_for_test(mut self, observer: &'input dyn Fn(PinnedWalkEventForTest)) -> Self {
        self.observer = Some(observer);
        self
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum PinnedWalkEventForTest {
    Entity(EntityId),
    Relation(crate::identity::data::RelationId),
    Adjacency(EntityId),
}

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// One global multi-source traversal. Success exhausts the frontier; no
    /// partial result escapes a stop. The returned owner retains its payer.
    pub fn complete_relation_walk(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedStructuralWalkScope<'_>,
    ) -> Result<PinnedStructuralWalk, PinnedStructuralWalkDenial> {
        let mut reads = PinnedStructuralWalkReads::default();
        let mut work = 0;
        // Final handoff may discard a completed value. Observations therefore
        // remain here, outside the native callback's returned payload.
        let result = execution.run_native_with_checkpoint(|context| {
            traversal::checkpoint(context, &scope, &mut work, 1)?;
            if !self.schema().entity_kinds.contains_key(&scope.entity_kind)
                || !self
                    .schema()
                    .relation_kinds
                    .contains_key(&scope.relation_kind)
            {
                return Err(PinnedStructuralWalkDenialKind::KindUnavailable);
            }
            let projection = VisibilityProjectionView::from_pinned(self);
            let memory = memory::WalkMemory::new(execution, context.has_accounting())?;
            traversal::execute(
                &projection,
                execution,
                &scope,
                context,
                &mut reads,
                &mut work,
                memory,
            )
        });
        match result {
            Ok((Ok(walk), _physical)) => Ok(walk),
            Ok((Err(kind), _physical)) => Err(PinnedStructuralWalkDenial { kind, reads, work }),
            Err(stop) => Err(PinnedStructuralWalkDenial {
                kind: PinnedStructuralWalkDenialKind::Read(
                    crate::visibility::materialization::read_records::read_request::cause(stop),
                ),
                reads,
                work,
            }),
        }
    }
}

#[cfg(test)]
mod long_walk;
#[cfg(test)]
mod tests;
