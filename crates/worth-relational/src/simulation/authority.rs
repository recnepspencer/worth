use crate::config::data::CompiledLanePolicy;
use crate::diagnostics::data::{
    DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticFields,
    RelationalDiagnosticValue, RelationalDiagnosticsEntry,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::CommitId;
use crate::identity::data::PartitionId;
use crate::runtime::RelationalRuntime;
use crate::simulation::data::{
    CompiledArtifactAuthorityStatus, CompiledArtifactError, CompiledExecutionArtifact,
    TopologyFreezeMode,
};

pub struct SimulationAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn simulation_authority(&self) -> SimulationAuthority<'_> {
        SimulationAuthority::new(self)
    }
}

impl<'runtime> SimulationAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub fn compile_execution_artifact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        partition_ids: Vec<PartitionId>,
    ) -> Result<CompiledExecutionArtifact, CompiledArtifactError> {
        crate::execution::check_request(execution).map_err(source_read_refused)?;
        ensure_compiled_lane_enabled(self.runtime)?;
        let envelope = source_commit_envelope(execution, self.runtime, commit_id)?;
        let artifact = build_compiled_execution_artifact(
            self.runtime.services.next_compiled_artifact_id(),
            commit_id,
            partition_ids,
            &envelope,
        );
        publish_compiled_execution_artifact(self.runtime, &artifact);
        Ok(artifact)
    }
}

fn ensure_compiled_lane_enabled(runtime: &RelationalRuntime) -> Result<(), CompiledArtifactError> {
    if runtime.config.execution.compiled_lane_policy == CompiledLanePolicy::DerivedCompiledLane {
        return Ok(());
    }

    Err(CompiledArtifactError {
        authority_status: CompiledArtifactAuthorityStatus::CompiledLaneDisabled,
        detail: "compiled execution lane is disabled for this profile".to_string(),
    })
}

fn source_commit_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit_id: CommitId,
) -> Result<std::sync::Arc<CanonicalCommitEnvelope>, CompiledArtifactError> {
    runtime
        .history()
        .demand_commit_envelope(execution, commit_id)
        .map_err(source_read_refused)?
        .ok_or_else(|| CompiledArtifactError {
            authority_status: CompiledArtifactAuthorityStatus::MissingSourceCommit,
            detail: format!("missing source commit {}", commit_id.0),
        })
}

fn source_read_refused(
    cause: crate::execution::RelationalExecutionDenialCause,
) -> CompiledArtifactError {
    CompiledArtifactError {
        authority_status: CompiledArtifactAuthorityStatus::ReadRequest(cause),
        detail: format!("compiled source read refused: {cause:?}"),
    }
}

fn build_compiled_execution_artifact(
    artifact_id: u64,
    commit_id: CommitId,
    partition_ids: Vec<PartitionId>,
    envelope: &CanonicalCommitEnvelope,
) -> CompiledExecutionArtifact {
    let partition_ids = partition_ids
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let compiled_record_count = envelope.patch.authoritative_record_patches.len();

    CompiledExecutionArtifact {
        artifact_id,
        source_commit_id: commit_id,
        source_version_id: envelope.commit.version_id,
        source_branch_id: envelope.branch_context.clone(),
        partition_ids: partition_ids.into_iter().collect(),
        topology_freeze_mode: TopologyFreezeMode::FreezeAtCommit,
        compiled_record_count,
    }
}

fn publish_compiled_execution_artifact(
    runtime: &RelationalRuntime,
    artifact: &CompiledExecutionArtifact,
) {
    runtime.services.store_compiled_artifact(artifact.clone());
    runtime.publication_authority().push_bounded_diagnostic(
        DiagnosticsScope::History,
        DiagnosticsArtifactKind::MinimalSummary,
        vec![compiled_execution_artifact_created(artifact)],
    );
}

fn compiled_execution_artifact_created(
    artifact: &CompiledExecutionArtifact,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::CommitPublished,
        "compiled execution artifact created",
        compiled_execution_artifact_created_fields(artifact),
    )
}

fn compiled_execution_artifact_created_fields(
    artifact: &CompiledExecutionArtifact,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "artifact_id",
            RelationalDiagnosticValue::Unsigned(artifact.artifact_id),
        ),
        (
            "source_commit_id",
            RelationalDiagnosticValue::CommitId(artifact.source_commit_id),
        ),
        (
            "source_version_id",
            RelationalDiagnosticValue::VersionId(artifact.source_version_id),
        ),
        (
            "source_branch_id",
            RelationalDiagnosticValue::BranchId(artifact.source_branch_id.clone()),
        ),
        ("partition_ids", partition_id_array(&artifact.partition_ids)),
        (
            "topology_freeze_mode",
            topology_freeze_mode_value(artifact.topology_freeze_mode),
        ),
        (
            "compiled_record_count",
            RelationalDiagnosticValue::unsigned(artifact.compiled_record_count),
        ),
    ])
    .into()
}

fn partition_id_array(partition_ids: &[PartitionId]) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        partition_ids
            .iter()
            .copied()
            .map(RelationalDiagnosticValue::PartitionId),
    )
}

fn topology_freeze_mode_value(freeze_mode: TopologyFreezeMode) -> RelationalDiagnosticValue {
    match freeze_mode {
        TopologyFreezeMode::FreezeAtCommit => RelationalDiagnosticValue::string("FreezeAtCommit"),
    }
}
