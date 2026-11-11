//! Immutable vocabulary exchanged by durable owner ports.

mod checkpoint_admission;
mod native_checkpoint_reference;
pub use native_checkpoint_reference::{
    NativeCheckpointBranchObservation, NativeCheckpointBranchReference,
    NativeCheckpointBranchTarget,
};
mod dispatch_custody;
pub use dispatch_custody::DispatchAttemptIdentity;
mod program_support;
mod workflow_ending_actor;
pub use program_support::{
    ProgramSupportObligationIdentity, ProgramSupportObligationKind, ProgramSupportStatus,
};
pub use workflow_ending_actor::{WorkflowEndingActorDecodeDenial, WorkflowEndingActorProvenance};
mod home_binding;
mod home_installation_transition;
pub use home_installation_transition::HomeProgramTransitionInstallationBinding;
mod home_program_revision;
mod home_program_transition;
pub use home_program_revision::HomeProgramRevisionBoundary;
mod home_program_step;
pub use home_program_step::HomeProgramTransitionProgramStep;
mod home_transition_caller;
mod home_transition_plan;
mod home_transition_target;
pub use checkpoint_admission::{
    CheckpointAdmissionCut, CheckpointAdmissionScope, CheckpointProductBindingIdentity,
};
pub use home_program_transition::{
    HomeProgramTransitionBodyDigest, HomeProgramTransitionIdentity,
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
};
pub use home_transition_caller::{
    HomeProgramTransitionCaller, HomeProgramTransitionCallerDenial, HomeProgramTransitionStepRole,
};
pub use home_transition_plan::{
    HomeProgramTransitionDescriptionDenial, HomeProgramTransitionPlanDescription,
};
pub use home_transition_target::{
    HomeProgramTransitionCatalogDigest, HomeProgramTransitionSchemaCarrierDigest,
    HomeProgramTransitionSchemaDigest, HomeProgramTransitionTargetMeaning,
};
mod world_identity;
mod world_journal;
pub use world_identity::*;
pub use world_journal::WorldDurableJournal;
mod composite_attempt_association;
pub use composite_attempt_association::{
    CompositeAttemptAssociation, CompositeAttemptAssociationOwnerMismatch,
};
mod home_identity;
mod home_incarnation;
mod owner_family;
mod query_output_settlement;
mod retention_class;
mod state_family;
pub use query_output_settlement::DurableQueryOutputSettlement;
pub use state_family::DurableStateFamily;
#[cfg(test)]
mod tests;

mod identity_seal {
    pub trait Sealed {}
}

/// Data whose identity is meaningful after its owning home is reopened.
///
/// This marker grants no runtime authority. Capabilities and process-local
/// identities cannot implement its private predecessor.
///
/// ```compile_fail,E0277
/// use worth_foundational::facade::DurableIdentity;
/// struct ProcessRuntimeOrdinal(u64);
/// impl DurableIdentity for ProcessRuntimeOrdinal {}
/// ```
pub trait DurableIdentity: identity_seal::Sealed {}

pub use home_binding::{
    ApplicationHomeBinding, ApplicationHomeBindingDescriptionDenial, HomeProgramBinding,
};
pub use home_identity::{HomeIdentity, HomeIdentityDenial};
pub use home_incarnation::{HomeIncarnation, HomeIncarnationExhausted};
pub use owner_family::{DurableOwnerFamily, UnsupportedDurableOwnerTag};
pub use retention_class::{DurableArtifactRetention, RetentionClass};
mod correction_material_pin;
pub use correction_material_pin::{
    CorrectionMaterialPinIdentity, CorrectionMaterialPinOwner, CorrectionMaterialPinTerminal,
};

mod world_original_fate;
pub use world_original_fate::WorldOriginalFate;

mod unclean_recovery;
pub use unclean_recovery::UncleanRecoveryUnsupported;
