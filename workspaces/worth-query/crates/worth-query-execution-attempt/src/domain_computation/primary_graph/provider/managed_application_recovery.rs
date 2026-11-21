pub mod performed;
pub mod producer;
pub use performed::WorthQueryManagedApplicationRecoveryPerformed;
pub use producer::ProducerRecoveryCustody;
use worth_runtime_world::facade::{
    RuntimeWorldRecoveryDenial, RuntimeWorldSettledRelationalAdoptionDenial,
};

use super::unpublished_idempotency::ManagedUnpublishedRecoveryStop;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryOperationAuthorizationDenial, WorthQueryOutputDemandDenial,
};
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};


pub enum RecoveredPublicationDelivery<'a> {
    Ordinary,
    ProgramSource,
    OwningSource,
    Producer(&'a mut dyn FnMut(ProducerRecoveryCustody)),
}

/// Owner result of an admitted recovery: performed publication, no effect, or
/// a new exact partial recovery replacing the previously retained handle.
pub enum WorthQueryManagedApplicationRecoveryOutcome {
    Performed(WorthQueryManagedApplicationRecoveryPerformed),
    /// World made no movement. The supplied exact recovery remains retryable.
    NoEffect,
    /// World issued a new partial; its provider claim was staged before the
    /// owner effect and now replaces the original exact recovery handle.
    ProductUnpublished {
        next: WorthQueryProductUnpublishedRecovery,
        prior_cleanup_failure: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    },
}

/// A refusal before another World owner effect. The borrowed original
/// recovery remains retained by its provider entry for a later admitted try.
#[derive(Debug)]
pub enum WorthQueryManagedApplicationRecoveryDenial {
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    ForeignAdmission,
    BindingMismatch,
    ProviderBusy,
    ProviderCapacity,
    ProviderAdmission(worth_relational::facade::mvcc::CompanionPreflightStop),
    World(RuntimeWorldRecoveryDenial),
    ProductUnavailable,
    Bridge(worth_runtime_bridge::facade::BridgeConditionalDenial),
    /// Rich World head-comparison evidence is retained only for this refusal.
    Adoption(Box<RuntimeWorldSettledRelationalAdoptionDenial>),
    Demand(WorthQueryOutputDemandDenial),
}

impl From<ManagedUnpublishedRecoveryStop> for WorthQueryManagedApplicationRecoveryDenial {
    fn from(stop: ManagedUnpublishedRecoveryStop) -> Self {
        match stop {
            ManagedUnpublishedRecoveryStop::Missing | ManagedUnpublishedRecoveryStop::Busy => {
                Self::ProviderBusy
            }
            ManagedUnpublishedRecoveryStop::Capacity => Self::ProviderCapacity,
        }
    }
}
