pub use crate::application::{
    WorthQueryConcurrentHostileMatrixArtifact, WorthQueryConcurrentHostileMatrixPosture,
    WorthQueryConcurrentHostileMatrixSabotage, WorthQueryConcurrentHostileMatrixSabotageKind,
    WorthQueryMilestoneClosureStatus, WorthQueryMilestoneNineSevenDerivedClosure,
    WorthQueryMilestoneNineSevenPhaseClosure, WorthQueryPublicBridgeForbiddenAccessFinding,
    WorthQueryPublicBridgeForbiddenAccessPattern,
    WorthQueryPublicBridgeProjectionConsumptionEvidence,
    WorthQueryPublicBridgePublishedProjectionReader, WorthQueryPublicBridgeReaderLaneCertification,
    WorthQueryPublicBridgeReaderLaneInventory, WorthQueryPublicBridgeReaderLanePosture,
    WorthQueryPublicBridgeReaderLaneSabotage, WorthQueryPublicBridgeReaderLaneSabotageKind,
    WorthQueryPublicBridgeReaderLaneSabotageOutcome,
};
pub use crate::basis_lifecycle::{
    basis_lifecycle_dx_certification_digest, basis_lifecycle_migration_audit,
    basis_lifecycle_migration_audit_digest, basis_lifecycle_phase_artifact_manifest_digest,
    basis_lifecycle_phase_manifest, basis_lifecycle_phase_progression_digest,
    basis_lifecycle_proof_shape_audit, basis_lifecycle_proof_shape_audit_digest,
    basis_lifecycle_public_boundary_audit, basis_lifecycle_public_boundary_audit_digest,
    basis_lifecycle_reuse_matrix, basis_lifecycle_reuse_matrix_digest,
    basis_lifecycle_support_matrix, basis_lifecycle_typestate_transition_digest,
    certify_basis_lifecycle, certify_basis_lifecycle_performance_slopes,
    BasisLifecycleCertificationBundle, BasisLifecycleCertificationLane,
    BasisLifecycleCertificationOutputDigest, BasisLifecycleCertificationOutputPosture,
    BasisLifecycleCertificationRow, BasisLifecycleMigrationAudit, BasisLifecycleMigrationAuditRow,
    BasisLifecycleMigrationCounters, BasisLifecycleMigrationPosture,
    BasisLifecycleMigrationSurface, BasisLifecyclePerformanceSlopeReport,
    BasisLifecyclePhaseArtifact, BasisLifecyclePhaseManifest, BasisLifecyclePhaseManifestRow,
    BasisLifecycleProofShapeAudit, BasisLifecycleProofShapeAuditRow,
    BasisLifecycleProofShapeEnforcement, BasisLifecycleProofShapeViolation,
    BasisLifecyclePublicBoundaryAudit, BasisLifecyclePublicBoundaryAuditRow,
    BasisLifecyclePublicBoundarySurface, BasisLifecycleReuseMatrix, BasisLifecycleReuseMatrixRow,
    BasisLifecycleReuseSurface, BasisLifecycleSlopeDigest, BasisLifecycleSlopeFamily,
};
pub use crate::domain_capabilities::{
    certify_domain_capabilities, worth_query_domain_capability_certification_output_manifest,
    worth_query_domain_capability_certification_surface,
    worth_query_domain_capability_public_surface_inventory,
    worth_query_domain_capability_representative_report,
    worth_query_domain_capability_slope_report, WorthQueryDomainCapabilityCertificationBundle,
    WorthQueryDomainCapabilityCertificationCounterSnapshot,
    WorthQueryDomainCapabilityCertificationOutput, WorthQueryDomainCapabilityCertificationSurface,
    WorthQueryDomainCapabilityCertifiedSurfaceInventory,
    WorthQueryDomainCapabilityCertifiedSurfaceRow,
};
pub use crate::domain_installation::certification_replay::{
    issue_query_certification_replay_capability, replay_installed_workflow,
    WorthQueryCertificationReplayAdmissionDenial, WorthQueryCertificationReplayCapability,
    WorthQueryCertificationReplayCounters, WorthQueryCertificationReplayOutcome,
    WorthQueryCertificationReplayResult, WorthQueryCertificationReplayStop,
    WorthQueryHistoricalBasisCorrespondence, WorthQueryHistoricalReplayAdmission,
    WorthQueryHistoricalReplayAdmissionDenial, WorthQueryReplayBasisRelationship,
};
pub use crate::domain_installation::historical_replay::{
    admit_installed_historical_replay_basis, replay_installed_workflow_historical,
    WorthQueryInstalledHistoricalReplayPath,
};
pub use crate::domain_installation::WorthQueryFoundationalReplayAttachment;
pub use crate::effect_lifecycle::{
    certify_effect_execution_pipeline, effect_lifecycle_public_surface_inventory,
    effect_lifecycle_support_matrix, EffectExecutionCertificationBundle,
    EffectExecutionCertificationLane, EffectExecutionCertificationOutputDigest,
    EffectExecutionCertificationRow,
};
pub use crate::identity_evolution::{
    IdentityEvolutionCertificationDenialEvidence, IdentityEvolutionCertificationEvidence,
    IdentityEvolutionCertificationResultEvidence,
};
pub use crate::integration_harness::public_bridge_hostile_certification::{
    compose_public_bridge_hostile_certification_digest,
    public_bridge_hostile_certification_evidence_label,
    public_bridge_hostile_published_artifact_component_digest,
    public_bridge_hostile_title_projection_artifacts,
    public_bridge_projection_artifacts_for_canonical_bundle,
    public_bridge_projection_artifacts_for_declarative_request,
    public_bridge_projection_artifacts_for_read_graph,
    PublicBridgeHostileCertificationComposeInput,
};
pub use crate::live::{LiveCertificationLane, LiveCertificationRejectionLane};
pub use crate::orchestration_inventory::{
    WorthQueryOrchestrationInventoryAudit, WorthQueryOrchestrationSurfaceCertificationReference,
};
pub use crate::policy_certification::{
    employee_record_policy_fixture, employee_record_policy_scale_report,
    policy_composition_parity_report, policy_identity_aware_inspector_parity_report,
    policy_mask_parity_report, policy_view_shape_parity_report, EmployeeRecordCertificationBundle,
    EmployeeRecordPolicyFixture, EmployeeRecordPolicyScenario, EmployeeRecordQueryFamily,
    EmployeeRecordTenantVariant, PolicyCompositionParityReport,
    PolicyIdentityAwareInspectorParityReport, PolicyMaskParityReport, PolicyScaleCounterSnapshot,
    PolicyScaleFixtureSize, PolicyScaleSlopeDigest, PolicyScaleSlopeReport,
    PolicyViewShapeParityReport,
};
pub use crate::policy_live::certify_policy_live_drift_evidence;
pub use crate::projection_consumption::{
    certify_consumed_projection_authority, certify_projection_consumption_closeout_core,
    consumed_projection_authority_support_matrix, projection_consumption_phase_progression_digest,
    projection_consumption_proof_shape_audit, projection_consumption_public_boundary_audit,
    projection_consumption_support_matrix, ConsumedProjectionAuthorityCertificationBundle,
    ConsumedProjectionAuthorityCertificationLane, ConsumedProjectionAuthorityCertificationRow,
    ProjectionConsumptionCertificationBundle, ProjectionConsumptionCertificationCounterSnapshot,
    ProjectionConsumptionCertificationLane, ProjectionConsumptionCertificationRow,
    ProjectionConsumptionCertifiedSourceSurface, ProjectionConsumptionProofShapeAudit,
    ProjectionConsumptionProofShapeAuditRow, ProjectionConsumptionPublicBoundaryAudit,
    ProjectionConsumptionPublicBoundaryAuditRow,
};
pub use crate::runtime::{
    build_causal_inspection_certification_scope, certify_causal_inspection_runtime_path,
    certify_intent_admission, certify_lower_runtime_non_bypass,
    certify_lower_runtime_performance_slopes, certify_lower_runtime_routing,
    inspect_lower_runtime_closeout, worth_query_intent_admission_certification_output_manifest,
    worth_query_intent_admission_closeout_extension_outputs,
    worth_query_intent_admission_legacy_parity_report, worth_query_intent_admission_mutation_audit,
    worth_query_intent_admission_oracle_report,
    worth_query_intent_admission_representative_family_report,
    worth_query_intent_admission_representative_output_report,
    worth_query_intent_admission_required_certification_outputs,
    worth_query_intent_admission_seeded_certification_report,
    worth_query_intent_admission_slope_report, worth_query_intent_admission_support_matrix,
    worth_query_lower_runtime_certification_output_manifest,
    worth_query_lower_runtime_closeout_extension_outputs,
    worth_query_lower_runtime_closeout_registry, worth_query_lower_runtime_closeout_report,
    worth_query_lower_runtime_closeout_report_digest,
    worth_query_lower_runtime_direct_import_audit,
    worth_query_lower_runtime_phase_artifact_manifest_digest,
    worth_query_lower_runtime_phase_manifest, worth_query_lower_runtime_phase_progression_digest,
    worth_query_lower_runtime_proof_shape_audit, worth_query_lower_runtime_proof_shape_digest,
    worth_query_lower_runtime_public_surface_inventory,
    worth_query_lower_runtime_required_certification_outputs,
    worth_query_lower_runtime_support_matrix, worth_query_lower_runtime_synthetic_tail_report,
    worth_query_lower_runtime_typestate_transition_digest, CausalInspectionBoundaryAudit,
    CausalInspectionCertificationBundle, CausalInspectionCertificationError,
    CausalInspectionCertificationErrorKind, CausalInspectionCertificationFailureEvidence,
    CausalInspectionCertificationFailureKind, CausalInspectionCertificationFailureSource,
    CausalInspectionCertificationLane, CausalInspectionCertificationScope,
    CausalInspectionPerformanceCertificationBundle, CausalInspectionProofShapeCertification,
    WorthQueryAspectApiFinalizationCloseout, WorthQueryAuthoritativeMutationEvidenceCloseout,
    WorthQueryDomainEvidenceCertificationBundle, WorthQueryDomainEvidenceCertificationSidecar,
    WorthQueryGraphReadPersistentArtifactAudit, WorthQueryIntentAdmissionCertificationBundle,
    WorthQueryIntentAdmissionCertificationCounterSnapshot,
    WorthQueryIntentAdmissionCertificationOutput, WorthQueryIntentAdmissionMutationAudit,
    WorthQueryIntentAdmissionMutationAuditRow, WorthQueryIntentAdmissionOracleManifestRow,
    WorthQueryIntentAdmissionProofShapeAudit, WorthQueryIntentAdmissionPublicBoundaryAudit,
    WorthQueryIntentAdmissionSeededCertificationReport, WorthQueryIntentAdmissionTopologyAudit,
    WorthQueryIntentAdmissionTopologyAuditRow, WorthQueryLowerRuntimeCertificationBundle,
    WorthQueryLowerRuntimeCertificationLane, WorthQueryLowerRuntimeCertificationOutputDigest,
    WorthQueryLowerRuntimeCertificationRow, WorthQueryLowerRuntimeCloseoutPosture,
    WorthQueryLowerRuntimeCloseoutRegistry, WorthQueryLowerRuntimeCloseoutReport,
    WorthQueryLowerRuntimeCloseoutRow, WorthQueryLowerRuntimeDirectImportAudit,
    WorthQueryLowerRuntimeDirectImportAuditRow, WorthQueryLowerRuntimeNonBypassAudit,
    WorthQueryLowerRuntimePhaseManifest, WorthQueryLowerRuntimePhaseManifestRow,
    WorthQueryLowerRuntimeProofShapeAudit, WorthQueryLowerRuntimeProofShapeAuditRow,
    WorthQueryReadCompositionPhaseOneCloseout,
};
pub use crate::subscription::{
    build_certified_family_coverage_handle, build_query_subscription_runtime_certification_scope,
    certify_query_subscription_activation, certify_query_subscription_runtime_family,
    certify_query_subscription_scale_slope, certify_subscription_lifecycle,
    CertificationCoverageReceipt, CertifiedFamilyCoverageHandle,
    QuerySubscriptionCertificationBundle, QuerySubscriptionCertificationDenialKind,
    QuerySubscriptionCertificationError, QuerySubscriptionLifecycleCloseoutSupport,
    QuerySubscriptionRuntimeCertificationBundle, QuerySubscriptionRuntimeCertificationCounters,
    QuerySubscriptionRuntimeCertificationError, QuerySubscriptionRuntimeCertificationErrorKind,
    QuerySubscriptionRuntimeCertificationScope, SubscriptionCertificationCoverageWidth,
    SubscriptionLifecycleCertificationBundle, SubscriptionLifecycleCertificationContext,
    SubscriptionLifecycleCertificationDenialKind, SubscriptionLifecycleCertificationError,
    SubscriptionLifecycleCloseout, SubscriptionLifecycleCloseoutKind,
    SubscriptionLifecyclePreviewCertification,
};
pub use worth_query_execution::facade::certification::{
    WorthQueryCertificationApplicationWork, WorthQueryCertificationCostObservation,
    WorthQueryCertificationCostRuntimeExt, WorthQueryCertificationCostScope,
    WorthQueryCertificationWorldHistory, WorthQueryCertificationWorldRetention,
};

/// Certification-only constructor for fixtures that compare a retained basis
/// with authority returned by an ordinary consumer journey.
pub fn resolve_runtime_current_snapshot_basis_for_certification(
    snapshot_identity: &crate::WorthQueryEvidenceIdentity,
    schema_basis_authority: crate::basis::QuerySchemaBasisAuthority,
) -> Result<crate::basis::ResolvedSnapshotBasis, crate::basis::BasisResolutionError> {
    crate::basis::resolve_runtime_current_snapshot_basis(
        snapshot_identity.clone(),
        schema_basis_authority,
    )
}

/// Admit an externally authored current-snapshot basis for an explicit
/// downstream certification fixture.
pub fn admit_runtime_current_snapshot_basis_for_certification(
    snapshot_identity: crate::WorthQueryEvidenceIdentity,
    schema_view: crate::basis::QuerySchemaView,
) -> Result<crate::basis::ResolvedSnapshotBasis, crate::basis::BasisResolutionError> {
    crate::basis::admit_runtime_current_snapshot_basis(snapshot_identity, schema_view)
}

/// Project a foundation-only projection outcome into the ordinary navigation
/// contract for certification fixtures that exercise warning and denial
/// postures which cannot be authored by ordinary callers.
pub fn ordinary_projection_outcome_for_certification(
    outcome: crate::projection_consumption::ProjectionAuthorityOutcome,
) -> crate::ordinary::read::WorthQueryProjectionOutcome {
    crate::ordinary::read::WorthQueryProjectionOutcome::from_foundation(outcome)
}

/// Apply the query-context advisory posture to an otherwise ordinary
/// projection outcome for downstream boundary fixtures.
pub fn ordinary_query_context_advisory_for_certification(
    outcome: crate::ordinary::read::WorthQueryProjectionOutcome,
) -> crate::ordinary::read::WorthQueryProjectionOutcome {
    outcome.with_query_context_advisory_for_certification()
}

/// Exercise a non-ordinary projection contract against an ordinary read
/// completion for downstream denial-boundary fixtures.
pub fn consume_projection_contract_for_certification(
    completion: &crate::ordinary::read::WorthQueryReadCompletion,
    contract: crate::projection_consumption::ProjectionAuthorityContract,
) -> crate::ordinary::read::WorthQueryProjectionOutcome {
    completion.consume_projection_contract_for_certification(contract)
}

/// Reconstruct the typed write-boundary receipt used by explicit downstream
/// certification fixtures. Ordinary consumers receive the cohesive mutation
/// outcome and do not assemble this lower-runtime artifact themselves.
pub fn write_authority_execution_receipt_for_certification(
    command: &crate::runtime::WorthQueryWriteCommand,
    receipt: crate::memory_workspace::WorthQueryMutationReceipt,
) -> crate::lower_runtime_routing::WriteAuthorityExecutionReceipt {
    crate::lower_runtime_routing::WriteAuthorityExecutionReceipt::from_command(command, receipt)
}

/// Runs the pre-primary-graph conditional classifier used by the installed
/// monolith. Phase 9 certification compares this real internal oracle with the
/// host path; ordinary consumers cannot reach it through an audience facade.
pub fn internal_conditional_outcome_for_certification(
    class: worth_signal::facade::SignalConditionalDecisionClass,
) -> crate::domain_installation::WorthQueryConditionalOutcomeClass {
    crate::domain_installation::classify_signal_decision(class)
}

/// Runs the Phase 9 host classifier against real Bridge evidence retained by
/// the pre-primary-graph oracle. This comparison is certification-only.
pub fn host_conditional_signal_for_certification(
    provenance: &crate::domain_installation::WorthQueryConditionalProvenance,
) -> worth_query_execution::facade::primary_graph::WorthQueryConditionalSignalDecision {
    worth_query_execution::facade::integration::classify_conditional_signal_for_certification(
        provenance.bridge_evidence(),
    )
}

pub use worth_query_execution::facade::primary_graph::WorthQueryConditionalSignalDecision as WorthQueryHostConditionalSignalDecisionForCertification;
