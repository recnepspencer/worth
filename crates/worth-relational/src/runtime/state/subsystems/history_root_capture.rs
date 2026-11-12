use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::branch::{
    PreparedRelationalBranchRootCapture, RelationalBranchRoot, RelationalBranchRootCaptureDenial,
};
use crate::history::data::CanonicalCommitEnvelope;

use super::HistorySubsystem;

impl HistorySubsystem {
    pub(crate) fn readmit_checkpoint_region_directory(
        &self,
        regions: worth_execution::ExecutionArray<crate::branch::RelationalRootRegionCheckpoint>,
        commit: crate::durability::checkpoint_admission::CheckpointAdmittedCommit,
        content_records: &crate::content::RelationalContentAttachments,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalBranchRootCaptureDenial> {
        RelationalBranchRoot::readmit_checkpoint_region_directory(
            &self.root_identity_issuer,
            regions,
            commit,
            content_records,
            request,
        )
    }
    pub(crate) fn readmit_branch_region_directory(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        content_records: &crate::content::RelationalContentAttachments,
        regions: worth_execution::ExecutionArray<crate::branch::RelationalRootRegionCheckpoint>,
        original: Arc<crate::branch::RelationalOriginalRootCommitEvidence>,
        descriptor: crate::branch::RelationalBranchRootDescriptor,
        schema: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalBranchRootCaptureDenial> {
        RelationalBranchRoot::readmit_region_directory(
            &self.root_identity_issuer,
            execution,
            content_records,
            regions,
            original,
            descriptor,
            schema,
        )
    }
    pub(crate) fn prepare_branch_root_capture<P: crate::storage::overlay::PartitionAccess>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: &P,
        published_delta: &crate::storage::RelationalPublishedPartitionDelta,
        previous: Option<&Arc<RelationalBranchRoot>>,
        envelope: Arc<CanonicalCommitEnvelope>,
        content: &crate::content::ContentPublication,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::runtime::RuntimeSymbolTable,
        execution_contracts: Option<Arc<crate::runtime::RelationalRootExecutionContracts>>,
    ) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
        #[cfg(test)]
        if self.root_capture_sabotage.swap(false, Ordering::Relaxed) {
            return Err(RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(
                crate::symbols::data::Symbol(u32::MAX),
            ));
        }
        RelationalBranchRoot::prepare_capture(
            execution,
            &self.root_identity_issuer,
            partitions,
            published_delta,
            previous,
            envelope,
            content,
            registry,
            symbols,
            execution_contracts,
        )
    }

    #[cfg(test)]
    pub(crate) fn sabotage_next_root_capture(&self) {
        self.root_capture_sabotage.store(true, Ordering::Relaxed);
    }

    pub(crate) fn readmit_branch_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        content_records: &crate::content::RelationalContentAttachments,
        partitions: BTreeMap<
            crate::identity::data::PartitionId,
            crate::storage::overlay::PartitionState,
        >,
        original: Arc<crate::branch::RelationalOriginalRootCommitEvidence>,
        descriptor: crate::branch::RelationalBranchRootDescriptor,
        schema_authority: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalBranchRootCaptureDenial> {
        RelationalBranchRoot::readmit(
            &self.root_identity_issuer,
            execution,
            content_records,
            partitions,
            original,
            descriptor,
            schema_authority,
            symbols,
        )
    }
}

impl HistorySubsystem {
    /// Initial image binding uses the actual catalog owner before considering encoding.
    /// Source-backed rows demand their admitted original once on this caller's request.
    pub(crate) fn original_root_evidence_for_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit: crate::history::data::CommitId,
    ) -> Result<
        Arc<crate::branch::RelationalOriginalRootCommitEvidence>,
        super::history_catalog_recovery::RelationalHistoryRecoveryDenial,
    > {
        use super::history_catalog_recovery::RelationalHistoryRecoveryDenial as Denial;
        let artifact = self
            .commit_artifact(commit)
            .ok_or_else(|| Denial::Invalid("root has no actual admitted catalog entry".into()))?;
        let residency = artifact.envelope_residency();
        if let Some(original) = residency.original_owner() {
            return Ok(original);
        }
        let positioned = residency
            .demand_positioned(execution, 1 << 30)
            .map_err(Denial::Source)?;
        crate::branch::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            positioned.canonical_arc().clone(),
        )
        .map_err(Denial::Root)
    }
}
