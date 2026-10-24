//! Installed application-aftermath classification and legal next actions.

mod canonical_basis;
mod correction_authority;
mod correction_binding;
pub(crate) use correction_binding::resolve_correction_binding;
pub use correction_binding::InstalledCorrectionBinding;
mod correction_mechanism;
mod denial;
mod external_effect_contract;
mod install;
mod install_validation;
mod lowering_correspondence;
mod next_action_contract;
mod owner_identity;
mod postcondition;
mod published_posture;
mod reconciliation;
mod recovery_contract;

#[cfg(test)]
mod tests;

pub use canonical_basis::WorthQueryAftermathCanonicalArtifact;
pub use correction_authority::InstalledCorrectionAuthority;
pub use correction_mechanism::{
    InstalledCompensation, InstalledCorrectionMechanism, InstalledLoweringCorrespondenceRef,
    InstalledPreImageDemand, InstalledPreImageLocus, InstalledRecordedInverse,
};
pub use denial::{
    WorthQueryAftermathInstallationDenial, WorthQueryAftermathInstallationDenialKind,
};
pub use external_effect_contract::{
    InstalledExternalEffectContract, InstalledExternalEffectPosture,
};
pub(crate) use install::install_application_aftermath;
pub use install::{WorthQueryInstalledAftermathContract, WorthQueryInstalledAftermathIdentity};
pub use lowering_correspondence::{
    AftermathLoweringCorrespondenceCatalog, InstalledLoweringCorrespondence,
    LoweringCorrespondenceResolutionDenial,
};
pub use next_action_contract::{
    CompensatableNextActionContract, CompensateNextAction, InstalledAftermathNextActionContract,
    IrreversibleNextActionContract, ReconcilableNextActionContract, ReconcileNextAction,
    ReversibleNextActionContract, UndoViaRecordedInverse,
};
pub use owner_identity::aftermath_owner_identity_digest;
pub use postcondition::InstalledAftermathPostcondition;
pub use published_posture::{derive_published_posture, PublishedAftermathPosture};
pub use reconciliation::WorthQueryInstalledReconciliationProcedure;
pub use recovery_contract::InstalledAftermathRecoveryContract;
