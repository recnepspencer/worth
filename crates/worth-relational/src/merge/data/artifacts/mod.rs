mod aspect_digest_rows;
mod correspondence_witness;
mod correspondence_witness_rows;
mod execution_authority_contract;
mod inspection_artifact;
mod planning_artifact_core;
mod planning_digest_basis;
mod proof_packet;
mod proof_packet_canonical;
mod schema_reconciliation_witness;
mod schema_reconciliation_witness_rows;
mod schema_reconciliation_witness_transition;
mod schema_snapshot_digest_basis;
mod strategy_witness;
mod strategy_witness_policy_rows;
mod strategy_witness_posture_rows;
mod witness_rows;
pub(crate) use witness_rows::MergeWitnessRows;

pub use aspect_digest_rows::{MergeLoweredAspectDigestRow, MergePolicyAspectDigestRow};
pub use correspondence_witness::RelationalMergeCorrespondenceWitness;
pub(crate) use correspondence_witness::{
    correspondence_posture_for_candidate, row_for_candidate, schema_declared_correspondence_posture,
};
pub use correspondence_witness_rows::{
    RelationalMergeCorrespondenceWitnessPosture, RelationalMergeCorrespondenceWitnessRow,
};
pub use execution_authority_contract::{
    MergeExecutionAuthorityContract, MergeExecutionAuthorizationRule,
    MergeExecutionConsumptionRule, MergeExecutionDecisionSurface,
};
pub use inspection_artifact::{
    RelationalMergeInspectionAdmission, RelationalMergeInspectionArtifact,
    RelationalMergeInspectionInput, RelationalMergeInspectionRow,
};
pub use planning_artifact_core::{MergePlanningArtifactCore, MergePlanningSummary};
pub use planning_digest_basis::{
    MergeArtifactDigestBasis, MergeBaseDigestBasis, MergeCausalDigestBasis,
    MergeConflictDigestBasis, MergeIdentityDigestBasis, MergeLoweredPlanDigestBasis,
    MergePolicyDigestBasis, MergeRequestDigestBasis,
};
pub use proof_packet::{
    RelationalMergeAdmittedSurfaceRow, RelationalMergeProofPacket,
    RelationalMergeProofPacketAdmissionPosture,
};
pub use proof_packet_canonical::RelationalMergeProofPacketCanonicalBasis;
pub use schema_reconciliation_witness::RelationalSchemaReconciliationWitness;
pub(crate) use schema_reconciliation_witness_rows::RelationalSchemaReconciliationWitnessRowInput;
pub use schema_reconciliation_witness_rows::{
    RelationalSchemaReconciliationBasisRow, RelationalSchemaReconciliationCorrespondenceLinkRow,
    RelationalSchemaReconciliationWitnessDenial, RelationalSchemaReconciliationWitnessPosture,
    RelationalSchemaReconciliationWitnessRow,
};
pub use schema_snapshot_digest_basis::{
    MergeSchemaKindClass, MergeSchemaKindSemanticSnapshot, MergeSchemaSnapshotDigestBasis,
};
pub use strategy_witness::RelationalMergeStrategyWitness;
pub use strategy_witness_policy_rows::RelationalMergeAspectPolicyWitnessRow;
pub use strategy_witness_posture_rows::{
    RelationalMergeDeletionStrategyWitnessRow, RelationalMergeTopologyStrategyWitnessRow,
};
