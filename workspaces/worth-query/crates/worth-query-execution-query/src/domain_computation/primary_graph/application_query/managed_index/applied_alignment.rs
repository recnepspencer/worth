//! A producer decision on an applied candidate reads a managed Parent at its
//! admitted before basis only while Native proves the candidate left that
//! Parent's Source unchanged. Otherwise the read refuses before any provider
//! call: the unpublished change is never hidden behind the committed image.
use super::execution::ProviderExecutionDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_execution::ExecutionRequest;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledManagedIndexParent};
use worth_relational::facade::{
    mvcc::RelationalAuthoredCoupledCandidate,
    runtime::{
        ManagedIndexSourcePreparationDenial, ManagedIndexSourceProgram,
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
    },
};

/// The live coupled assembly that issued the applied candidate a producer
/// decision reads. Only it can certify what its unpublished prefix changed.
pub trait AppliedManagedSourceAlignment {
    fn managed_source_unchanged_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<bool, ManagedIndexSourcePreparationDenial>;
}

impl AppliedManagedSourceAlignment for RelationalAuthoredCoupledCandidate<'_> {
    fn managed_source_unchanged_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<bool, ManagedIndexSourcePreparationDenial> {
        Self::managed_source_unchanged_at(self, execution, program, candidate)
    }
}

/// One applied decision read's posture: the issuing assembly and the exact
/// candidate image the decision reads.
#[derive(Clone, Copy)]
pub struct AppliedManagedIndexRead<'a> {
    pub alignment: &'a dyn AppliedManagedSourceAlignment,
    pub candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
}

impl AppliedManagedIndexRead<'_> {
    /// Refuses unless the candidate left the Parent's Source unchanged, so the
    /// admitted before basis is exactly the candidate's Source image.
    pub fn require_aligned<S: ApplicationSchema>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        association: &WorthQueryInstalledManagedIndexParent,
    ) -> Result<(), Denial> {
        let graph = application
            .runtime
            .primary_graph()
            .ok_or(Denial::ForeignAssociation)?;
        let program = graph
            .layout_ref()
            .managed_source(&association.definition().parts().name)
            .ok_or(Denial::ForeignAssociation)?;
        if self
            .alignment
            .managed_source_unchanged_at(execution, program, self.candidate)
            .map_err(Denial::NativeSource)?
        {
            Ok(())
        } else {
            Err(Denial::UnpublishedSourceChange)
        }
    }
}
