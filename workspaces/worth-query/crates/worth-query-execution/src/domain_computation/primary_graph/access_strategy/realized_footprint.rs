//! Retained consistency authority exposes no rows, READ grant or publication.
mod comparison;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Stop;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_proof::{RealizedFootprintAuthority, VerifiedRealizedFootprint};
use worth_relational::facade::{
    runtime::{ManagedIndexSourceObservation, RelationalRuntime},
    snapshots::SnapshotHandle,
};

/// Which already captured descriptive evidence the caller compares.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum CapturedInfluenceComparison {
    Content,
    SelectedMembership,
}

/// Only the installed managed adapter implements this consistency execution.
pub(in crate::domain_computation::primary_graph) trait RetainedManagedInfluence:
    Send + Sync
{
    fn as_any(&self) -> &dyn std::any::Any;
    fn comparison_work(&self) -> u64;
    fn content(&self) -> worth_foundational::facade::ProjectedCandidateContentV1;
    fn audience(&self) -> [u8; 32];
    fn same_inputs(
        &self,
        other: &dyn RetainedManagedInfluence,
        purpose: CapturedInfluenceComparison,
        checkpoint: &mut dyn FnMut(u64) -> bool,
    ) -> bool;
    fn revalidate(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        audience: &CapturedManagedInfluence,
    ) -> Result<Option<RevalidatedManagedSource>, Stop>;
}

pub(in crate::domain_computation::primary_graph) struct RevalidatedManagedSource {
    pub(in crate::domain_computation::primary_graph) observation:
        Arc<ManagedIndexSourceObservation>,
    pub(in crate::domain_computation::primary_graph) proof: VerifiedRealizedFootprint,
    pub(in crate::domain_computation::primary_graph) allocation: Arc<SourceFactsAllocation>,
}

pub(in crate::domain_computation::primary_graph) struct CapturedManagedInfluence {
    proof: VerifiedRealizedFootprint,
    authority: RealizedFootprintAuthority,
    support: Arc<dyn RetainedManagedInfluence>,
    _allocation: Arc<SourceFactsAllocation>,
}
/// This exact target and its legality witness stay together until Native pinning.
pub(in crate::domain_computation::primary_graph) struct CheckedRealizedSource {
    observation: Arc<ManagedIndexSourceObservation>,
    _proof: VerifiedRealizedFootprint,
    _allocation: Arc<SourceFactsAllocation>,
}
impl CheckedRealizedSource {
    pub(in crate::domain_computation::primary_graph) fn native(
        &self,
    ) -> &ManagedIndexSourceObservation {
        &self.observation
    }
}
impl CapturedManagedInfluence {
    pub(in crate::domain_computation::primary_graph) fn comparison_work(&self) -> u64 {
        self.support.comparison_work()
    }
    pub(in crate::domain_computation::primary_graph) fn content(
        &self,
    ) -> worth_foundational::facade::ProjectedCandidateContentV1 {
        self.support.content()
    }
    pub(in crate::domain_computation::primary_graph) fn audience(&self) -> [u8; 32] {
        self.support.audience()
    }
    /// A false checkpoint cannot be swallowed into success by an erased callee.
    pub(in crate::domain_computation::primary_graph) fn same_inputs<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        self.compare(other, CapturedInfluenceComparison::Content, admit)
    }
    pub(in crate::domain_computation::primary_graph) fn same_selected_membership<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        self.compare(
            other,
            CapturedInfluenceComparison::SelectedMembership,
            admit,
        )
    }
    fn compare<E>(
        &self,
        other: &Self,
        purpose: CapturedInfluenceComparison,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        admit(self.comparison_work())?;
        admit(other.comparison_work())?;
        comparison::with_checkpoint(admit, &mut |checkpoint| {
            self.authority.admits(&other.proof)
                && other.authority.admits(&self.proof)
                && self
                    .support
                    .same_inputs(other.support.as_ref(), purpose, checkpoint)
        })
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        proof: VerifiedRealizedFootprint,
        authority: RealizedFootprintAuthority,
        support: Arc<dyn RetainedManagedInfluence>,
        allocation: Arc<SourceFactsAllocation>,
    ) -> Result<Self, Stop> {
        if !authority.admits(&proof) {
            return Err(Stop::Unavailable);
        }
        Ok(Self {
            proof,
            authority,
            support,
            _allocation: allocation,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn revalidate(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) -> Result<Option<CheckedRealizedSource>, Stop> {
        execution
            .consult()
            .map_err(|cause| Stop::Execution(cause.into()))?;
        if !self.authority.admits(&self.proof) {
            return Err(Stop::Unavailable);
        }
        let Some(RevalidatedManagedSource {
            observation,
            proof,
            allocation,
        }) = self
            .support
            .revalidate(runtime, execution, snapshot, self)?
        else {
            return Ok(None);
        };
        if !self.authority.admits(&proof) {
            return Err(Stop::Unavailable);
        }
        Ok(Some(CheckedRealizedSource {
            observation,
            _proof: proof,
            _allocation: allocation,
        }))
    }
}
