mod backup_cut_source_verification;
#[cfg(test)]
mod checkpoint_backup_tests;
mod checkpoint_backup_verification;
mod manifest_semantic_validation;
mod owner_artifact_verification;
mod owner_denial_classification;
mod owner_family_mapping;
mod owner_media_read;
mod owner_resource_budget;
mod owner_semantic_verification;
mod structurally_verified_bundle;
mod verification_budget;
mod verification_owned_memory;
mod verification_report;
mod verification_support;
mod verify_backup;

pub use backup_cut_source_verification::{
    verify_backup_cut_sources, verify_backup_cut_sources_with_cancellation,
    BackupCutSourceVerificationDenial, BackupCutSourceVerificationReport,
};
pub use checkpoint_backup_verification::{
    checkpoint_backup_frontier_digest, verify_bounded_checkpoint_backup_artifact_from_reader,
    BoundedCheckpointBackupDenial, BoundedCheckpointBackupObservation,
    BoundedCheckpointBackupVerificationRequest,
};
pub use structurally_verified_bundle::StructurallyVerifiedBackupBundle;
pub use verification_budget::BackupVerificationBudget;
pub(crate) use verification_report::BackupVerificationReportEvidence;
pub use verification_report::{
    BackupArtifactSemanticDefectKind, BackupVerificationDefect, BackupVerificationReadAccounting,
    BackupVerificationReport,
};
pub(crate) use verify_backup::verify_staged_materialized_backup;
pub use verify_backup::{
    verify_materialized_backup, verify_materialized_backup_with_cancellation,
    BackupStructuralVerificationDenial, BackupVerificationAllocationPhase,
};
mod bounded_physical_artifact;
pub use bounded_physical_artifact::{
    verify_bounded_extent_artifact_from_reader, verify_bounded_page_artifact_from_reader,
    verify_bounded_root_manifest_artifact_from_reader, BoundedPhysicalArtifactDenial,
    BoundedPhysicalArtifactObservation, VerifiedRootManifestArtifact,
};
