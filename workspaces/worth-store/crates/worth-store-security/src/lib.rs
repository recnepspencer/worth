#![forbid(unsafe_code)]

mod authenticity;
mod authority_source;
mod raw_security_declarations;
mod recovery_metadata;
#[cfg(test)]
mod recovery_metadata_tests;
mod repair_blast_radius;
mod scope;
mod scope_vocabulary;
#[cfg(test)]
mod security_authority_source_tests;
mod security_metadata;
mod security_metadata_canonical;
#[cfg(test)]
mod security_metadata_tests;
mod trust_boundary;
#[cfg(test)]
mod vocabulary_tests;

pub use authenticity::authenticity_check::{
    StoreAuthenticityCheck, StoreAuthenticityCheckInput, StorePhysicalAuthenticityCheck,
    StoreScopedAuthenticityCheck,
};
pub use authenticity::authenticity_counters::StoreAuthenticityCheckCounterSnapshot;
pub use authenticity::authenticity_denial::{
    StoreAuthenticityCheckDenial, StoreAuthenticityCheckDenialKind,
};
pub use authenticity::authenticity_result::{StoreAuthenticityResult, StoreAuthenticityResultKind};
pub use authenticity::authenticity_vocabulary::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass,
};
pub use authenticity::authenticity_witness::{
    admit_store_authenticity_witness_observation, StoreAuthenticityWitnessBinding,
    StoreAuthenticityWitnessInput, StoreAuthenticityWitnessObservationDeclaration,
};
pub use authority_source::{
    classify_app_org_id_as_security_scope_source, classify_audit_record_as_security_scope_source,
    classify_foundational_evidence_as_security_scope_source,
    classify_iam_role_as_security_scope_source,
    classify_identity_provider_claim_as_security_scope_source,
    classify_kms_key_id_as_security_scope_source,
    classify_offline_verifier_evidence_as_security_scope_source,
    classify_operator_identity_as_security_scope_source,
    classify_proof_progression_as_security_scope_source,
    classify_raw_string_as_security_scope_source, classify_semantic_id_as_security_scope_source,
    classify_store_current_authority_as_security_scope_source,
    classify_terminal_json_label_as_security_scope_source, StoreSecurityAuthoritySource,
};
pub use raw_security_declarations::{
    evaluate_deserialized_security_scope_readmission,
    readmit_deserialized_security_scope_declaration, StoreApplicationOrgIdClaim, StoreIamRoleClaim,
    StoreJwtSubjectClaim, StoreKmsKeyIdentifier, StoreOperatorIdentityClaim,
    StoreRawSecurityScopeDeclaration, StoreRepairAuditRecordClaim,
    StoreSecurityScopeDeclarationProvenance, StoreSecurityScopeReadmissionEvaluation,
};
pub use recovery_metadata::{
    RecoveryCheckpointRecordSecurityMetadataEnvelope,
    RecoveryCheckpointRecordSecurityMetadataIdentity,
    RecoveryCheckpointRecordSecurityMetadataSource, RecoveryRootSecurityMetadataAdmission,
    RecoveryRootSecurityMetadataEnvelope, RecoverySecurityScopePropagation,
    RecoverySecurityScopePropagationCounters, RecoverySecurityScopePropagationDenial,
    RecoverySecurityScopePropagationInput, RecoveryWalRecordSecurityMetadataEnvelope,
    RecoveryWalRecordSecurityMetadataIdentity, RecoveryWalRecordSecurityMetadataSource,
};
pub use repair_blast_radius::{
    reject_repair_authority_source, repair_blast_radius_authenticity,
    repair_blast_radius_expectation, StoreRepairPhysicalRegionAdmissionOutcome,
    StoreRepairPhysicalRegionDeclaration, StoreRepairPhysicalRegionWitness,
};
pub use scope::layout_partition::{
    admit_layout_partition_security_scope, StoreLayoutPartitionSecurityWitness,
};
pub use scope::security_scope_admission::{
    admission_counter_snapshot, admit_store_security_scope,
    evaluate_store_security_scope_admission, StoreSecurityScopeAdmissionEvaluation,
    StoreSecurityScopeAdmissionOutcome,
};
pub use scope::security_scope_admission_basis::{
    StoreSecurityScopeAdmissionBasis, StoreSecurityScopeAdmissionExpectation,
};
pub use scope::security_scope_admission_denial::{
    StoreSecurityScopeAdmissionDeferred, StoreSecurityScopeAdmissionDenial,
    StoreSecurityScopeAdmissionFailure, StoreSecurityScopeAdmissionRebindRequired,
    StoreSecurityScopeAdmissionStale,
};
pub use scope::security_scope_admission_request::StoreSecurityScopeAdmissionRequest;
pub use scope::security_scope_counters::StoreSecurityScopeAdmissionCounterSnapshot;
pub use scope::security_scope_custody_readmission::{
    admit_readmitted_trust_boundary_security_scope,
    readmit_trust_boundary_security_scope_declaration, StoreReadmittedSecurityScope,
};
pub use scope::security_scope_denial::{
    reject_non_store_security_scope_source, StoreSecurityScopeDenial, StoreSecurityScopeDenialKind,
};
pub use scope::security_scope_identity::StoreSecurityScopeIdentity;
pub use scope::security_scope_propagation::{
    deny_drifted_store_security_scope, deny_missing_store_security_scope,
    deny_stale_store_security_scope, propagate_store_security_scope,
    StoreSecurityScopePropagationCounters, StoreSecurityScopePropagationDenial,
    StoreSecurityScopePropagationDenialKind, StoreSecurityScopePropagationOutcome,
    StoreSecurityScopePropagationSite, StoreSecurityScopePropagationWitness,
};
pub use scope::security_scope_receipt::{
    StoreAuthorityBoundSecurityScopeReceipt, StoreSecurityScopeAdmissionReceipt,
    StoreSecurityScopeAdmissionReceiptId, StoreSecurityScopeProofProgressionIdentity,
};
pub use scope::security_scope_roles::{
    StoreSecurityEvidenceVocabulary, StoreSecurityReadinessVocabulary,
    StoreSecurityReadinessVocabularyTerm, StoreSecurityRequirementVocabulary,
    StoreSecurityResultVocabulary, StoreSecurityWitnessVocabulary,
    StoreSecurityWitnessVocabularyTerm,
};
#[cfg(any(test, feature = "certification-test-authority"))]
pub use scope::security_scope_test_authority::{
    admitted_security_scope_for_identity_for_test,
    admitted_store_internal_security_scope_for_io_qos_test,
    admitted_store_internal_security_scope_for_named_physical_witness_test,
    admitted_store_internal_security_scope_for_physical_witness_test,
    admitted_store_managed_root_security_scope_for_layout_partition_test,
    admitted_store_wal_checkpoint_security_scope_for_layout_partition_test,
    admitted_tenant_artifact_security_scope_for_layout_partition_test,
    admitted_tenant_page_export_prepared_scope_for_layout_partition_test,
    admitted_tenant_page_security_scope_for_layout_partition_test,
    admitted_tenant_page_without_authenticity_for_layout_partition_test,
    admitted_tenant_wal_checkpoint_security_scope_for_layout_partition_test,
    admitted_wrong_io_qos_security_scope_for_test, readmitted_foreign_wal_security_scope_for_test,
    readmitted_wal_security_scope_for_test,
};
pub use scope::security_scope_witnesses::{
    StoreAdmittedSecurityScope, StoreCurrentAuthenticityScopeWitness,
    StoreCurrentCustodyScopeWitness, StoreCurrentKeyScopeWitness,
    StoreCurrentKeyVersionScopeWitness, StoreCurrentSecurityScopeWitnessSet,
    StoreCurrentTenantScopeWitness,
};
pub use scope_vocabulary::{
    StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture, StoreLegacySecurityPosture,
    StoreTenantScope,
};
pub use security_metadata::{
    admit_store_security_metadata, StoreRawSecurityMetadataDeclaration,
    StoreRawSecurityMetadataProjection, StoreSecurityMetadata,
    StoreSecurityMetadataAdmissionDenial, StoreSecurityMetadataAdmissionInput,
    StoreSecurityMetadataProjectionSource,
};
pub use security_metadata_canonical::{
    compare_store_security_metadata, StoreSecurityMetadataCanonicalBasis,
};
pub use trust_boundary::trust_boundary_observation::{
    store_backup_restore_boundary_fact, store_custody_domain_boundary_fact,
    store_deployment_boundary_fact, store_instance_boundary_fact,
    store_key_scope_generation_boundary_fact, store_offline_transfer_boundary_fact,
    store_tenant_scope_authority_boundary_fact,
};
pub use trust_boundary::{
    StoreBackupRestoreAfterKeyRotationBoundaryEvidence,
    StoreBackupRestoreAfterKeyRotationBoundaryFact, StoreBackupRestoreBoundaryFactInput,
    StoreCustodyDomainBoundaryEvidence, StoreCustodyDomainBoundaryFact,
    StoreCustodyDomainBoundaryFactInput, StoreDeploymentBoundaryFact,
    StoreDifferentDeploymentBoundaryEvidence, StoreDifferentDeploymentBoundaryFact,
    StoreDifferentStoreInstanceBoundaryEvidence, StoreDifferentStoreInstanceBoundaryFact,
    StoreKeyScopeGenerationBoundaryEvidence, StoreKeyScopeGenerationBoundaryFact,
    StoreKeyScopeGenerationBoundaryFactInput, StoreOfflineExportImportBoundaryEvidence,
    StoreOfflineExportImportBoundaryFact, StoreOfflineTransferBoundaryFact,
    StoreStoreInstanceBoundaryFact, StoreTenantScopeAuthorityBoundaryEvidence,
    StoreTenantScopeAuthorityBoundaryFact, StoreTenantScopeAuthorityBoundaryFactInput,
    StoreTrustBoundaryCrossing, StoreTrustBoundaryCrossingEvidence, StoreTrustBoundaryEvidence,
    StoreTrustBoundaryEvidenceDenial, StoreTrustBoundaryReadmissionTrigger,
};
