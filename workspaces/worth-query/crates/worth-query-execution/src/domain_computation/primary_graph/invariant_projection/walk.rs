//! Complete structural reads through the existing snapshot projection authority.
use worth_execution::{MapKernelContext, WorkCeilingDenial};
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema};
use worth_relational::facade::{
    identity::EntityId,
    runtime::{
        PinnedStructuralWalk, PinnedStructuralWalkDenialKind, PinnedStructuralWalkReads,
        PinnedStructuralWalkScope,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};

use super::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};

pub(super) mod memory;
mod observation;
mod result;
pub(in crate::domain_computation::primary_graph) use observation::{
    StructuralWalkKey, StructuralWalkObservation,
};
#[cfg(test)]
mod tests;
pub use result::{
    WorthQueryInvariantProjectionWalk, WorthQueryInvariantProjectionWalkIter,
    WorthQueryInvariantProjectionWalkStep,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Complete reachability, including the root at distance zero.
    pub fn walk<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>, Denial> {
        self.closure(relation, direction, std::slice::from_ref(root))
    }

    /// One global multi-root walk, ordered by minimum distance then identity.
    pub fn closure<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryInvariantEntityIdentity<Schema, Entity>],
    ) -> Result<WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>, Denial> {
        let execution = *self.execution;
        let result = execution.run_native_with_checkpoint(|context| {
            let native = self.read_complete_walk(&relation, direction, roots, context)?;
            self.realized_scope
                .admit_walk(native.visits(), &execution, context)
                .map_err(|cause| self.walk_stop(cause, relation.name()))?;
            let observation =
                StructuralWalkObservation::issue(native, relation.name(), direction, self, context)
                    .map_err(|cause| self.walk_stop(cause, relation.name()))?;
            Ok(WorthQueryInvariantProjectionWalk::new(
                observation,
                roots.first(),
            ))
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.walk_stop(Cause::from(stop), relation.name())),
        }
    }

    /// Forward reflexive reachability. Both identities must belong to this reader.
    pub fn reaches<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        from: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        to: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<bool, Denial> {
        if !self.identity_is_local(to, relation.to()) || to.kind != from.kind {
            return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
        }
        let execution = *self.execution;
        let subject = relation.name();
        let result = execution.run_native_with_checkpoint(|context| {
            let walk = self.walk(relation, RelationDirection::Forward, from)?;
            for visit in walk.observation.native().visits() {
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

    fn read_complete_walk<Relation, Entity>(
        &mut self,
        relation: &ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryInvariantEntityIdentity<Schema, Entity>],
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<PinnedStructuralWalk, Denial> {
        let layout = self
            .layout
            .relation(relation.name())
            .cloned()
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
        if layout.from != layout.to {
            return Err(Denial::new(Kind::EndpointUnavailable, relation.name()));
        }
        for root in roots {
            self.walk_checkpoint(context, relation.name())?;
            if !self.identity_is_local(root, relation.from()) || root.kind != layout.from {
                return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
            }
        }
        let bytes = roots
            .len()
            .checked_mul(std::mem::size_of::<EntityId>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| self.walk_stop(Cause::ChargedBytesOverflow, relation.name()))?;
        // The lowering vector drops before this original temporary reservation.
        let _memory = memory::reserve(self.execution, context.has_accounting(), bytes)
            .map_err(|cause| self.walk_stop(cause, relation.name()))?;
        let mut seeds = Vec::new();
        seeds
            .try_reserve_exact(roots.len())
            .map_err(|_| self.walk_stop(Cause::AllocationFailed, relation.name()))?;
        for root in roots {
            self.walk_checkpoint(context, relation.name())?;
            seeds.push(root.entity_id);
        }
        let pin = self
            .runtime
            .read_truth()
            .pin_read_capability(self.snapshot, &[], self.execution)
            .map_err(|denial| match denial {
                worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(
                    stop,
                ) => self.walk_stop(
                    Cause::from(WorkCeilingDenial::Stopped(stop)),
                    relation.name(),
                ),
                _ => Denial::new(Kind::EndpointUnavailable, relation.name()),
            })?;
        let native = pin.complete_relation_walk(
            self.execution,
            PinnedStructuralWalkScope::new(&seeds, layout.from, layout.kind, direction)
                .with_work_limit(self.work_budget.remaining()),
        );
        let (reads, work) = match &native {
            Ok(walk) => (walk.reads(), walk.work_units()),
            Err(denial) => (denial.reads, denial.work),
        };
        self.record_walk_reads(reads);
        if !self.work_budget.consume(work) {
            return Err(Denial::new(Kind::WorkBudgetExceeded, relation.name()));
        }
        native.map_err(|denial| match denial.kind {
            PinnedStructuralWalkDenialKind::Read(cause) => self.walk_stop(cause, relation.name()),
            PinnedStructuralWalkDenialKind::WorkLimitExceeded => {
                self.work_budget.mark_exceeded();
                Denial::new(Kind::WorkBudgetExceeded, relation.name())
            }
            _ => Denial::new(Kind::EndpointUnavailable, relation.name()),
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

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn walk_stop(
        &mut self,
        cause: Cause,
        subject: &str,
    ) -> Denial {
        self.native_read_stop.get_or_insert(cause);
        Denial::new(
            Kind::ReadExecution(crate::facade::primary_graph::relational_execution_kind(
                cause, None,
            )),
            subject,
        )
    }
}
