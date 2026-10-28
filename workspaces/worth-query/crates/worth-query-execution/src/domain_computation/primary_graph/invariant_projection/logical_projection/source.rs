//! Pinned source acquisition preserves original work, request stops and backing.
use super::*;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_relational::facade::{
    runtime::{
        DerivedRelationReadDenial as NativeDenial, PinnedDerivedRelationRead,
        RelationalPinnedReadAdmissionDenial,
    },
    schema::{DerivedRelationDenial as DefinitionDenial, NativeMemberKey},
};

impl<S: ApplicationSchema> Reader<'_, S> {
    pub(super) fn read_logical_source<R, F, T>(
        &mut self,
        relation: ApplicationDerivedRelationRef<S, R, F, T>,
        root: NativeMemberKey,
        shape: Shape,
    ) -> Result<(Arc<PinnedDerivedRelationRead>, Arc<SourceFactsAllocation>), Denial> {
        self.projection_member_domains(relation)?;
        let definition = self
            .layout
            .derived_relation(relation.name())
            .expect("checked installed source")
            .handle
            .clone();
        let allocation = SourceFactsAllocation::open(self.execution)
            .map_err(|cause| self.walk_stop(cause, relation.name()))?;
        allocation
            .claim(
                self.execution,
                (size_of::<PinnedDerivedRelationRead>() + 2 * size_of::<usize>()) as u64,
            )
            .map_err(|cause| self.walk_stop(cause, relation.name()))?;
        let native = {
            let pin = self
                .runtime
                .read_truth()
                .pin_read_capability(self.snapshot, &[], self.execution)
                .map_err(|cause| match cause {
                    RelationalPinnedReadAdmissionDenial::Request(stop) => {
                        self.walk_stop(stop, relation.name())
                    }
                    _ => Denial::new(Kind::EndpointUnavailable, relation.name()),
                })?;
            pin.derived_source(
                self.execution,
                &definition,
                root,
                shape,
                self.work_budget.remaining(),
            )
        }
        .map_err(|cause| self.logical_source_denial(cause, relation.name()))?;
        let reads = native.data().reads();
        self.work
            .record_adjacencies(reads.adjacency_lists, reads.relation_records);
        self.work.record_endpoints(reads.entity_records);
        if !self.work_budget.consume(native.work_units()) {
            self.work_budget.mark_exceeded();
            return Err(Denial::new(Kind::WorkBudgetExceeded, relation.name()));
        }
        for (entity, _) in native.data().entity_dependencies() {
            self.execution
                .check_live()
                .map_err(|cause| self.walk_stop(cause.into(), relation.name()))?;
            self.realized_scope.record(*entity);
        }
        self.execution
            .check_live()
            .map_err(|cause| self.walk_stop(cause.into(), relation.name()))?;
        Ok((Arc::new(native), allocation))
    }

    fn logical_source_denial(&mut self, cause: NativeDenial, subject: &str) -> Denial {
        match cause {
            NativeDenial::Request(cause) => self.walk_stop(cause, subject),
            NativeDenial::Definition(DefinitionDenial::WorkBudgetExceeded) => {
                self.work_budget.mark_exceeded();
                Denial::new(Kind::WorkBudgetExceeded, subject)
            }
            NativeDenial::Definition(
                DefinitionDenial::ForeignDefinition | DefinitionDenial::ForeignObservation,
            ) => Denial::new(Kind::ForeignIdentity, subject),
            NativeDenial::Structural(cause) => Denial::new(Kind::DerivedSource(cause), subject),
            NativeDenial::Definition(_) => Denial::new(Kind::EndpointUnavailable, subject),
        }
    }
}
