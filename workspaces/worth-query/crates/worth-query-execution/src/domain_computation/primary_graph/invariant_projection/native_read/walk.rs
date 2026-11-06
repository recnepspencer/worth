//! Structural walk execution consumes borrowed root facts without schema instantiation.
use super::super::{
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use super::*;
use crate::domain_computation::primary_graph::structural_walk::{
    memory, StructuralWalkObservation, StructuralWalkQuery,
};
use worth_execution::{MapKernelContext, WorkCeilingDenial};
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::{
    runtime::{PinnedStructuralWalkDenialKind, PinnedStructuralWalkReads},
    transactions::RelationalExecutionDenialCause as Cause,
};
pub(in crate::domain_computation::primary_graph::invariant_projection) trait WalkRoots {
    fn root_count(&self) -> usize;
    fn get(&self, index: usize) -> NativeIdentity<'_>;
}
pub(in crate::domain_computation::primary_graph::invariant_projection) struct TypedRoots<'a, S, E>(
    pub &'a [WorthQueryInvariantEntityIdentity<S, E>],
);
impl<S, E> WalkRoots for TypedRoots<'_, S, E> {
    fn root_count(&self) -> usize {
        self.0.len()
    }
    fn get(&self, index: usize) -> NativeIdentity<'_> {
        self.0[index].native_identity()
    }
}
struct OneRoot<'a>(NativeIdentity<'a>);
impl WalkRoots for OneRoot<'_> {
    fn root_count(&self) -> usize {
        1
    }
    fn get(&self, _index: usize) -> NativeIdentity<'_> {
        self.0
    }
}
impl NativeRead<'_, '_> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn closure(
        &mut self,
        relation: &str,
        entity: &str,
        direction: RelationDirection,
        roots: &dyn WalkRoots,
    ) -> Result<Arc<StructuralWalkObservation>, Denial> {
        let execution = *self.execution;
        let result = execution.run_native_with_checkpoint(|context| {
            let observation =
                self.read_complete_walk(relation, entity, direction, roots, context)?;
            self.scope
                .admit_walk(observation.native().visits(), &execution, context)
                .map_err(|cause| self.walk_stop(cause, relation))?;
            Ok(observation)
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.walk_stop(Cause::from(stop), relation)),
        }
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn reaches(
        &mut self,
        relation: &str,
        source: &str,
        destination: &str,
        from: NativeIdentity<'_>,
        to: NativeIdentity<'_>,
    ) -> Result<bool, Denial> {
        if !self.local(to, destination) || to.kind != from.kind {
            return Err(Denial::new(Kind::ForeignIdentity, relation));
        }
        let execution = *self.execution;
        let subject = relation;
        let result = execution.run_native_with_checkpoint(|context| {
            let walk =
                self.closure(relation, source, RelationDirection::Forward, &OneRoot(from))?;
            for visit in walk.native().visits() {
                self.walk_checkpoint(context, subject)?;
                if visit.entity.entity_id == to.entity_id {
                    return Ok(true);
                }
            }
            Ok(false)
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.walk_stop(Cause::from(stop), subject)),
        }
    }
    fn read_complete_walk(
        &mut self,
        relation: &str,
        entity: &str,
        direction: RelationDirection,
        roots: &dyn WalkRoots,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<StructuralWalkObservation>, Denial> {
        let layout = self
            .layout
            .relation(relation)
            .cloned()
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation))?;
        if layout.from != layout.to {
            return Err(Denial::new(Kind::EndpointUnavailable, relation));
        }
        for index in 0..roots.root_count() {
            let root = roots.get(index);
            self.walk_checkpoint(context, relation)?;
            if !self.local(root, entity) || root.kind != layout.from {
                return Err(Denial::new(Kind::ForeignIdentity, relation));
            }
        }
        let bytes = roots
            .root_count()
            .checked_mul(std::mem::size_of::<EntityId>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| self.walk_stop(Cause::ChargedBytesOverflow, relation))?;
        // The lowering vector drops before this original temporary reservation.
        let _memory = memory::reserve(self.execution, context.has_accounting(), bytes)
            .map_err(|cause| self.walk_stop(cause, relation))?;
        let mut seeds = Vec::new();
        seeds
            .try_reserve_exact(roots.root_count())
            .map_err(|_| self.walk_stop(Cause::AllocationFailed, relation))?;
        for index in 0..roots.root_count() {
            let root = roots.get(index);
            self.walk_checkpoint(context, relation)?;
            seeds.push(root.entity_id);
        }
        let pin = self
            .runtime
            .read_truth()
            .pin_read_capability(self.snapshot, &[], self.execution)
            .map_err(|denial| match denial {
                worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(
                    stop,
                ) => self.walk_stop(stop, relation),
                _ => Denial::new(Kind::EndpointUnavailable, relation),
            })?;
        let observed = StructuralWalkObservation::read(
            &pin,
            self.execution,
            StructuralWalkQuery {
                relation,
                roots: &seeds,
                entity_kind: layout.from,
                relation_kind: layout.kind,
                direction,
                maximum_work: self.budget.remaining(),
                control: None,
            },
            context,
        );
        let (reads, work) = match &observed {
            Ok(walk) => (walk.native().reads(), walk.native().work_units()),
            Err(denial) => (denial.reads, denial.work),
        };
        self.record_walk_reads(reads);
        if !self.budget.consume(work) {
            return Err(Denial::new(Kind::WorkBudgetExceeded, relation));
        }
        observed.map_err(|denial| match denial.kind {
            PinnedStructuralWalkDenialKind::Read(cause) => self.walk_stop(cause, relation),
            PinnedStructuralWalkDenialKind::WorkLimitExceeded => {
                self.budget.mark_exceeded();
                Denial::new(Kind::WorkBudgetExceeded, relation)
            }
            _ => Denial::new(Kind::EndpointUnavailable, relation),
        })
    }
    fn record_walk_reads(&mut self, reads: PinnedStructuralWalkReads) {
        self.work
            .record_adjacencies(reads.adjacency_lists, reads.relation_records);
        self.work.record_endpoints(reads.entity_records);
    }
    fn walk_checkpoint(
        &mut self,
        context: &mut MapKernelContext<'_, '_>,
        subject: &str,
    ) -> Result<(), Denial> {
        let admitted = if context.has_accounting() {
            context.checkpoint(1)
        } else {
            context.poll()
        };
        admitted
            .map_err(|stop| self.walk_stop(Cause::from(WorkCeilingDenial::Stopped(stop)), subject))
    }
}
