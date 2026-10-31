//! Framework-owned traversal completes before any candidate result is returned.
mod refinement;
mod traversal;
pub(in crate::domain_computation::primary_graph) use traversal::visit;
mod frame;
mod frontier;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    SourceTree, WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexSourceSlot as Slot,
    WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Denial,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
pub(in crate::domain_computation::primary_graph) use frame::CoverageFrame;
pub(in crate::domain_computation::primary_graph) use frontier::RecordedFrontier;
use std::{any::Any, sync::Arc};
use worth_execution::{ExecutionArray, ExecutionRequest};
use worth_proof::{ManagedIndexCoverageAuthority, ManagedIndexCoverageIssuer};

pub(in crate::domain_computation::primary_graph) struct CoverageVerifier {
    issuer: ManagedIndexCoverageIssuer,
    authority: ManagedIndexCoverageAuthority,
}
impl CoverageVerifier {
    pub(in crate::domain_computation::primary_graph) fn new() -> Self {
        let issuer = ManagedIndexCoverageIssuer::fresh();
        let authority = issuer.authority();
        Self { issuer, authority }
    }
    pub(in crate::domain_computation::primary_graph) fn authority(
        &self,
    ) -> &ManagedIndexCoverageAuthority {
        &self.authority
    }
    pub(in crate::domain_computation::primary_graph) fn complete<'a, 'loan>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        frame: CoverageFrame<'a, 'loan>,
        tree: &SourceTree,
        frontier: RecordedFrontier,
        pruning: bool,
        slots: ExecutionArray<Slot>,
        control: &Control<'_>,
    ) -> Result<CandidateSelection<'a, 'loan>, Denial> {
        let proof = frontier.verify(&self.issuer, tree, pruning, control)?;
        frontier.admits_slots(
            tree,
            slots.elements(),
            proof.matched_source_count(),
            control,
        )?;
        let allocation = SourceFactsAllocation::open(execution)
            .map_err(|cause| control.refuse(Stop::MaterialStorage(cause)))?;
        allocation
            .claim_arc::<RetainedCoverage>(execution)
            .map_err(|cause| control.refuse(Stop::MaterialStorage(cause)))?;
        control.poll()?;
        Ok(CandidateSelection {
            frame,
            slots,
            coverage: Some(Arc::new(RetainedCoverage {
                proof,
                frontier,
                nodes: Arc::clone(frame.nodes),
                refinement: None,
                _allocation: allocation,
            })),
        })
    }
}

/// Only the receiver transition releases slots; captured evidence has no public door.
pub(in crate::domain_computation::primary_graph) struct CandidateSelection<'a, 'loan> {
    frame: CoverageFrame<'a, 'loan>,
    slots: ExecutionArray<Slot>,
    coverage: Option<Arc<RetainedCoverage>>,
}
impl<'a, 'loan> CandidateSelection<'a, 'loan> {
    pub(in crate::domain_computation::primary_graph) fn conservative(
        frame: CoverageFrame<'a, 'loan>,
        slots: ExecutionArray<Slot>,
    ) -> Self {
        Self {
            frame,
            slots,
            coverage: None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn receive(
        self,
        expected: CoverageFrame<'_, '_>,
        authority: &ManagedIndexCoverageAuthority,
        complete: bool,
        control: &Control<'_>,
    ) -> Result<ReceivedSelection, Denial> {
        control.poll()?;
        if !self.frame.same_invocation(&expected)
            || self.coverage.as_ref().is_some_and(|coverage| {
                !coverage.admits(authority) || !Arc::ptr_eq(&coverage.nodes, expected.nodes)
            })
        {
            return Err(control.refuse(Stop::ForeignCoverage).into());
        }
        if (complete || expected.requires_complete()) && self.coverage.is_none() {
            return Err(control.refuse(Stop::IncompleteCoverage).into());
        }
        if expected.posture == worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Candidates
            && self.coverage.as_ref().is_some_and(|coverage| coverage.refinement.is_some()) {
            return Err(control.refuse(Stop::ForeignCoverage).into());
        }
        if expected.posture == worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined
            && !self.coverage.as_ref().is_some_and(|coverage| coverage.refinement.is_some()) {
            return Err(control.refuse(Stop::IncompleteRefinement).into());
        }
        Ok(ReceivedSelection {
            slots: self.slots,
            coverage: self.coverage,
        })
    }
}
pub(in crate::domain_computation::primary_graph) struct ReceivedSelection {
    slots: ExecutionArray<Slot>,
    coverage: Option<Arc<RetainedCoverage>>,
}
impl ReceivedSelection {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (ExecutionArray<Slot>, Option<Arc<RetainedCoverage>>) {
        (self.slots, self.coverage)
    }
    pub(in crate::domain_computation::primary_graph) fn complete_slots(
        &self,
    ) -> Result<&[Slot], Stop> {
        if self.coverage.is_none() {
            return Err(Stop::IncompleteCoverage);
        }
        Ok(self.slots.elements())
    }
}
pub(in crate::domain_computation::primary_graph) struct RetainedCoverage {
    refinement: Option<refinement::RetainedRefinement>,
    proof: worth_proof::AdmittedManagedIndexCoverage,
    frontier: RecordedFrontier,
    nodes: Arc<dyn Any + Send + Sync>,
    _allocation: Arc<SourceFactsAllocation>,
}
impl RetainedCoverage {
    pub(in crate::domain_computation::primary_graph) fn admits(
        &self,
        authority: &ManagedIndexCoverageAuthority,
    ) -> bool {
        authority.admits(&self.proof)
    }
}
impl std::fmt::Debug for RetainedCoverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedCoverage")
            .field("regions", &self.frontier.len())
            .finish_non_exhaustive()
    }
}
