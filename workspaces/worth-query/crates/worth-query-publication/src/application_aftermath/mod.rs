//! Closed consumer projections of sealed application-aftermath outcomes.

mod access_and_disclosure;
mod boundary_evidence;
mod external_effect;
mod outcome;
mod recovery;

pub use access_and_disclosure::{publish_application_aftermath, publish_recovery_support};
pub use boundary_evidence::{
    WorthQueryPublishedApplicationCommitBoundaryEvidence, WorthQueryPublishedApplicationCommitKind,
    WorthQueryPublishedCanonicalWork, WorthQueryPublishedMutationWork,
};
pub use external_effect::{
    WorthQueryPublishedExternalEffectFailure, WorthQueryPublishedExternalEffectPosture,
    WorthQueryPublishedExternalEffectPostureKind,
    WorthQueryPublishedUnsupportedProtocolVersionPosture,
};
pub use outcome::{WorthQueryPublishedAftermathPosture, WorthQueryPublishedApplicationAftermath};
pub use recovery::{
    WorthQueryPublishedRecoveryDurability, WorthQueryPublishedRecoverySupport,
    WorthQueryPublishedRecoverySupportTruth,
};
