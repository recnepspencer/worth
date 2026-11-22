//! Native readiness is selected before any producer input admission or handler.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

/// An unconsumed selection keeps its actual candidate and any Native proof.
/// It has not registered that proof as a used input of the final publication.
pub struct SelectedCoupledSource {
    pub read: execution::CoupledSourceRead,
    pub pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    pub fn select_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before_pin: &RelationalPinnedRead,
        demand: &DeclaredCoupledOutputDemand,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<SelectedCoupledSource, CoupledSourceReadStop> {
        Ok(SelectedCoupledSource {
            read: self.read_source_one(execution, before_pin, demand, candidate)?,
            pin: candidate.pin_source(execution)?,
        })
    }

    pub fn read_source_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before_pin: &RelationalPinnedRead,
        demand: &DeclaredCoupledOutputDemand,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<execution::CoupledSourceRead, CoupledSourceReadStop> {
        let principal = self.principal.ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "coupled producer authentication",
            )
        })?;
        let source_entry = self
            .application
            .installed_producers
            .declared_coupled_source(demand)?;
        source_entry.coupled_executor().read_coupled_source(
            execution,
            self.application,
            principal,
            self.request,
            self.before,
            before_pin,
            demand,
            candidate,
            if self.superseding.get() {
                execution::CoupledSourceReadPurpose::Supersede
            } else {
                execution::CoupledSourceReadPurpose::OutputPreparation
            },
        )
    }
}
