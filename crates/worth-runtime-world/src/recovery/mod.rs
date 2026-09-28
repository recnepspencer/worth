mod catalog;
mod cleanup;
mod continuation;
mod product_unpublished;
mod progress;

pub use cleanup::ProductUnpublishedCleanup;
pub use continuation::{ProductUnpublishedNextAction, RecoveryContinuationContract};
pub use product_unpublished::{
    ProductUnpublishedCause, ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle,
    ProductUnpublishedRetentionPosture,
};

/// Why preparing a product publication that adopts a settled Relational
/// commit, retained by a `ProductUnpublished` recovery record, was refused.
///
/// Returned by `RuntimeWorldRecoveryPort::prepare_settled_relational_adoption`.
/// Neither variant publishes anything; the recovery record stays retained.
#[derive(Debug)]
pub enum RuntimeWorldSettledRelationalAdoptionDenial {
    /// The recovery record could not be used: the owner was unavailable, the
    /// record could not be inspected, or it carries no settled Relational
    /// adoption.
    Recovery(RuntimeWorldRecoveryDenial),
    /// Preparing the composite publication was refused with no effect.
    Publication(crate::publication::NoEffectCompositePublication),
}

pub(crate) use catalog::{RecoveryCatalog, RecoveryCatalogDenial, ReservedProductUnpublishedSlot};
pub(crate) use cleanup::RecoveryCleanupOutcome;

#[cfg(test)]
pub(crate) use product_unpublished::next_actions_for_progress;
pub(crate) use product_unpublished::ProductUnpublishedOwnerEffectsRecord;
pub(crate) use product_unpublished::RetainedAttemptFacts;
pub(crate) use progress::ProductUnpublishedLiveObligations;

mod denial;
pub use denial::RuntimeWorldRecoveryDenial;

mod performed_denial;
pub use performed_denial::PerformedPublicationRecoveryDenial;
