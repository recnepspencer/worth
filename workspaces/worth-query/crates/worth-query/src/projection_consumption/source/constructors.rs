use crate::canonicalization::CanonicalResultShapeArtifact;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::query_context::{QueryContextExecutionArtifact, QueryContextExecutionFamily};
use crate::runtime::{
    WorthQueryDerivedArtifactBinding, WorthQueryDerivedMaterializationTarget,
    WorthQueryLiveArtifactBinding, WorthQueryLiveArtifactTarget, WorthQueryLiveReadReceipt,
    WorthQueryMutationTargetClass, WorthQueryReadExecutionEngine, WorthQueryReadReceipt,
    WorthQueryWriteReceipt,
};
use worth_runtime_bridge::facade::{
    BridgeGroupedTruthViewArtifact, BridgeMaterializedRowSetArtifact, TruthSnapshotIdentity,
};
use worth_runtime_bridge::facade::{
    RelationalAuthoritativeRowSetArtifact, RelationalGroupedProjectionArtifact,
};

use super::{
    ProjectionConsumptionSource, ProjectionSourceBasisAuthority, ProjectionSourceCapabilityProfile,
    ProjectionSourceExecutionPosture, ProjectionSourceFamily, ProjectionSourceIdentity,
    ProjectionSourceReferenceIdentity, ProjectionWriteReceiptCapabilities,
};

impl ProjectionConsumptionSource {
    pub fn from_read_receipt(
        receipt: &WorthQueryReadReceipt,
        result_shape: &CanonicalResultShapeArtifact,
    ) -> Self {
        Self {
            source_reference_identities: Vec::new(),
            family: ProjectionSourceFamily::QueryReadReceipt,
            capability_profile: ProjectionSourceCapabilityProfile::QueryReadReceipt {
                execution_posture: execution_posture_from_read_engine(receipt.execution_engine()),
            },
            query_digest: Some(receipt.canonical_query_digest().to_string()),
            basis_digest: Some(receipt.basis_digest().to_string()),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                receipt.snapshot_identity().clone(),
            ),
            result_digest: Some(receipt.result_digest().to_string()),
            result_shape_digest: Some(result_shape.digest().as_str().to_string()),
            source_identity: ProjectionSourceIdentity::from_evidence_identity(
                receipt.read_graph_identity().clone(),
            ),
            materialized_fact_posture: receipt.materialized_fact_posture().cloned(),
        }
    }

    pub fn from_live_read_receipt(receipt: &WorthQueryLiveReadReceipt) -> Self {
        Self {
            family: ProjectionSourceFamily::QueryLiveReadReceipt,
            capability_profile: ProjectionSourceCapabilityProfile::QueryLiveReadReceipt {
                execution_posture: ProjectionSourceExecutionPosture::Current,
            },
            query_digest: Some(receipt.canonical_query_digest().to_string()),
            basis_digest: Some(receipt.snapshot_evidence_identity().as_str().to_string()),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                receipt.snapshot_identity().clone(),
            ),
            result_digest: Some(receipt.result_digest().to_string()),
            result_shape_digest: Some(receipt.view_shape_digest().to_string()),
            source_identity: ProjectionSourceIdentity::from_evidence_identity(
                receipt.installation_identity().clone(),
            ),
            source_reference_identities: Vec::new(),
            materialized_fact_posture: receipt.materialized_fact_posture().cloned(),
        }
    }

    pub fn from_write_receipt(receipt: &WorthQueryWriteReceipt) -> Self {
        let resolved_target = receipt.target_evidence().resolved();
        let mut source_reference_identities = Vec::new();
        if let Some(provenance) = receipt.provenance_evidence() {
            source_reference_identities.push(ProjectionSourceReferenceIdentity::synthetic(
                "bridge_provenance_execution_record",
                provenance.execution_record_digest().as_str(),
            ));
        }
        if let Some(symbolic_reference) = receipt.symbolic_target_reference_evidence() {
            source_reference_identities.push(ProjectionSourceReferenceIdentity::synthetic(
                "symbolic_target_reference",
                symbolic_reference.symbol().as_str(),
            ));
        }
        Self {
            family: ProjectionSourceFamily::QueryWriteReceipt,
            capability_profile: ProjectionSourceCapabilityProfile::QueryWriteReceipt {
                capabilities: ProjectionWriteReceiptCapabilities {
                    has_target_identity: receipt.target_entity_identity().is_some(),
                    has_source_reference: receipt.provenance_evidence().is_some()
                        || receipt.symbolic_target_reference_evidence().is_some(),
                    has_effect_continuity: receipt.continuity_mutation_evidence().is_some(),
                    has_relation_endpoint: matches!(
                        resolved_target.target_class(),
                        WorthQueryMutationTargetClass::Entity
                    ) && resolved_target.collection().is_some()
                        && resolved_target.entity_identity().is_some(),
                },
            },
            query_digest: None,
            basis_digest: Some(receipt.snapshot_evidence_identity().as_str().to_string()),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                receipt.snapshot_identity().clone(),
            ),
            result_digest: None,
            result_shape_digest: None,
            source_identity: ProjectionSourceIdentity::from_evidence_identity(
                receipt.commit_evidence_identity().clone(),
            ),
            source_reference_identities,
            materialized_fact_posture: None,
        }
    }

    pub fn from_query_context_execution(execution: &QueryContextExecutionArtifact) -> Self {
        let mut source_reference_identities = Vec::new();
        if let Some(materialization_path_identity) = execution.materialization_path_identity() {
            source_reference_identities.push(ProjectionSourceReferenceIdentity::synthetic(
                "query_context_materialization_path",
                materialization_path_identity,
            ));
        }
        if let Some(preview_provenance_identity) = execution.preview_provenance_identity() {
            source_reference_identities.push(ProjectionSourceReferenceIdentity::synthetic(
                "query_context_preview_provenance",
                preview_provenance_identity,
            ));
        }
        Self {
            source_reference_identities,
            family: ProjectionSourceFamily::QueryContextExecution,
            capability_profile: ProjectionSourceCapabilityProfile::QueryContextExecution {
                execution_posture: execution_posture_from_query_context_family(execution.family()),
            },
            query_digest: Some(execution.query_digest().to_string()),
            basis_digest: Some(execution.basis_digest().to_string()),
            basis_authority: ProjectionSourceBasisAuthority::query_context(
                execution.family().clone(),
                execution.basis_digest(),
            ),
            result_digest: Some(execution.result_digest().to_string()),
            result_shape_digest: Some(execution.result_shape_digest().to_string()),
            source_identity: ProjectionSourceIdentity::artifact(
                execution
                    .materialization_path_identity()
                    .unwrap_or_else(|| execution.family().as_str()),
            ),
            materialized_fact_posture: execution.materialized_fact_posture().cloned(),
        }
    }

    pub fn from_relational_row_set(row_set: &RelationalAuthoritativeRowSetArtifact) -> Self {
        Self {
            source_reference_identities: Vec::new(),
            family: ProjectionSourceFamily::RelationalRowSet,
            capability_profile: ProjectionSourceCapabilityProfile::RelationalRowSet,
            query_digest: None,
            basis_digest: Some(snapshot_identity_digest(row_set.snapshot_identity())),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                snapshot_identity_authority(row_set.snapshot_identity()),
            ),
            result_digest: None,
            result_shape_digest: None,
            source_identity: ProjectionSourceIdentity::artifact(row_set.digest().as_str()),
            materialized_fact_posture: None,
        }
    }

    pub fn from_relational_grouped_projection(
        grouped_projection: &RelationalGroupedProjectionArtifact,
    ) -> Self {
        Self {
            source_reference_identities: Vec::new(),
            family: ProjectionSourceFamily::RelationalGroupedProjection,
            capability_profile: ProjectionSourceCapabilityProfile::RelationalGroupedProjection,
            query_digest: None,
            basis_digest: Some(snapshot_identity_digest(
                grouped_projection.snapshot_identity(),
            )),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                snapshot_identity_authority(grouped_projection.snapshot_identity()),
            ),
            result_digest: None,
            result_shape_digest: None,
            source_identity: ProjectionSourceIdentity::artifact(
                grouped_projection.digest().as_str(),
            ),
            materialized_fact_posture: None,
        }
    }

    pub fn from_bridge_truth_view_row_set(row_set: &BridgeMaterializedRowSetArtifact) -> Self {
        Self {
            source_reference_identities: Vec::new(),
            family: ProjectionSourceFamily::BridgeTruthViewRowSet,
            capability_profile: ProjectionSourceCapabilityProfile::BridgeTruthViewRowSet,
            query_digest: None,
            basis_digest: Some(snapshot_identity_digest(row_set.basis_snapshot_identity())),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                snapshot_identity_authority(row_set.basis_snapshot_identity()),
            ),
            result_digest: None,
            result_shape_digest: None,
            source_identity: ProjectionSourceIdentity::artifact(row_set.digest().as_str()),
            materialized_fact_posture: None,
        }
    }

    pub fn from_bridge_grouped_truth_view(
        grouped_truth_view: &BridgeGroupedTruthViewArtifact,
    ) -> Self {
        Self {
            source_reference_identities: Vec::new(),
            family: ProjectionSourceFamily::BridgeGroupedTruthView,
            capability_profile: ProjectionSourceCapabilityProfile::BridgeGroupedTruthView,
            query_digest: None,
            basis_digest: Some(snapshot_identity_digest(
                grouped_truth_view.basis_snapshot_identity(),
            )),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                snapshot_identity_authority(grouped_truth_view.basis_snapshot_identity()),
            ),
            result_digest: None,
            result_shape_digest: None,
            source_identity: ProjectionSourceIdentity::artifact(
                grouped_truth_view.digest().as_str(),
            ),
            materialized_fact_posture: None,
        }
    }

    pub fn from_retained_derived_artifact_binding(
        binding: &WorthQueryDerivedArtifactBinding,
    ) -> Self {
        Self {
            source_reference_identities: retained_derived_target_references(
                "retained_target_view",
                binding.targets(),
            ),
            family: ProjectionSourceFamily::RetainedDerivedArtifactBinding,
            capability_profile: ProjectionSourceCapabilityProfile::RetainedDerivedArtifactBinding,
            query_digest: None,
            basis_digest: Some(
                binding
                    .snapshot_identity()
                    .evidence_identity()
                    .as_str()
                    .to_string(),
            ),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                binding.snapshot_identity().clone(),
            ),
            result_digest: None,
            result_shape_digest: None,
            source_identity: ProjectionSourceIdentity::artifact(binding.binding_for_reporting()),
            materialized_fact_posture: None,
        }
    }

    pub fn from_live_artifact_binding(binding: &WorthQueryLiveArtifactBinding) -> Self {
        Self {
            source_reference_identities: retained_live_target_references(
                "live_target_view",
                binding.targets(),
            ),
            family: ProjectionSourceFamily::LiveArtifactBinding,
            capability_profile: ProjectionSourceCapabilityProfile::LiveArtifactBinding,
            query_digest: None,
            basis_digest: Some(
                binding
                    .snapshot_identity()
                    .evidence_identity()
                    .as_str()
                    .to_string(),
            ),
            basis_authority: ProjectionSourceBasisAuthority::runtime_snapshot(
                binding.snapshot_identity().clone(),
            ),
            result_digest: None,
            result_shape_digest: None,
            source_identity: ProjectionSourceIdentity::artifact(binding.binding_digest()),
            materialized_fact_posture: None,
        }
    }
}

fn snapshot_identity_digest(snapshot_identity: &TruthSnapshotIdentity) -> String {
    snapshot_identity_authority(snapshot_identity)
        .evidence_identity()
        .as_str()
        .to_string()
}

fn snapshot_identity_authority(
    snapshot_identity: &TruthSnapshotIdentity,
) -> WorthQuerySnapshotIdentity {
    let parts = snapshot_identity
        .relational_snapshot_parts()
        .expect("projection consumption source must carry typed relational snapshot identity");
    WorthQuerySnapshotIdentity::from_relational_snapshot(parts)
}

pub(crate) fn execution_posture_from_read_engine(
    engine: &WorthQueryReadExecutionEngine,
) -> ProjectionSourceExecutionPosture {
    match engine {
        WorthQueryReadExecutionEngine::QueryRuntimeCurrent => {
            ProjectionSourceExecutionPosture::Current
        }
        WorthQueryReadExecutionEngine::QueryRuntimeBranch => {
            ProjectionSourceExecutionPosture::Branch
        }
        WorthQueryReadExecutionEngine::QueryRuntimeHistorical => {
            ProjectionSourceExecutionPosture::Historical
        }
        WorthQueryReadExecutionEngine::QueryRuntimePreviewDerived => {
            ProjectionSourceExecutionPosture::PreviewDerived
        }
    }
}

pub(crate) fn execution_posture_from_query_context_family(
    family: &QueryContextExecutionFamily,
) -> ProjectionSourceExecutionPosture {
    match family {
        QueryContextExecutionFamily::RuntimeCurrent => ProjectionSourceExecutionPosture::Current,
        QueryContextExecutionFamily::RuntimeBranch => ProjectionSourceExecutionPosture::Branch,
        QueryContextExecutionFamily::HistoricalMaterialized => {
            ProjectionSourceExecutionPosture::Historical
        }
        QueryContextExecutionFamily::PreviewDerivedHistorical => {
            ProjectionSourceExecutionPosture::PreviewDerived
        }
    }
}

fn retained_derived_target_references<'a>(
    label: &'static str,
    targets: impl IntoIterator<Item = &'a WorthQueryDerivedMaterializationTarget>,
) -> Vec<ProjectionSourceReferenceIdentity> {
    targets
        .into_iter()
        .map(|target| {
            ProjectionSourceReferenceIdentity::synthetic(
                label,
                target.terminal_view_name_projection(),
            )
        })
        .collect()
}

fn retained_live_target_references<'a>(
    label: &'static str,
    targets: impl IntoIterator<Item = &'a WorthQueryLiveArtifactTarget>,
) -> Vec<ProjectionSourceReferenceIdentity> {
    targets
        .into_iter()
        .map(|target| {
            ProjectionSourceReferenceIdentity::synthetic(
                label,
                target.terminal_view_name_projection(),
            )
        })
        .collect()
}
