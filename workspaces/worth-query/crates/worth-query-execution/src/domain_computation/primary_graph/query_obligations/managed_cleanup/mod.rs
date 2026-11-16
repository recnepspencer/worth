//! Recorded logical cleanup and the actual live owner's resource release.
use crate::domain_computation::{
    WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunTerminalKind,
    WorthQueryMutationCleanupFailureKind,
};
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductUnpublishedOwnerEffectsIdentity,
    ManagedCleanupObligationIdentity,
};

pub(super) mod codec;
pub(super) mod fold;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryManagedCleanupObligation {
    pub identity: ManagedCleanupObligationIdentity,
    pub run: String,
    pub failure: WorthQueryMutationCleanupFailureKind,
    /// World remains the authority for this original's business outcome.
    pub original: Option<QueryManagedCleanupOriginal>,
    pub release: Option<QueryManagedCleanupRelease>,
}

/// DATA recorded only from the actual native cleanup owner's receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryManagedCleanupRelease {
    pub terminal: WorthQueryManagedRunTerminalKind,
    pub disposition: WorthQueryManagedRunCleanupDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryManagedCleanupRecord {
    Retain(QueryManagedCleanupObligation),
    Released {
        identity: ManagedCleanupObligationIdentity,
        release: QueryManagedCleanupRelease,
    },
    Failed {
        identity: ManagedCleanupObligationIdentity,
        failure: WorthQueryMutationCleanupFailureKind,
    },
    Discharge {
        identity: ManagedCleanupObligationIdentity,
    },
}

/// Descriptive references to distinct World owner-issued identity families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryManagedCleanupOriginal {
    Attempt(DurableCompositePublicationAttemptIdentity),
    Unpublished(DurableProductUnpublishedOwnerEffectsIdentity),
}
impl QueryManagedCleanupOriginal {
    pub(super) fn home(self) -> worth_foundational::facade::HomeIdentity {
        match self {
            Self::Attempt(original) => original.owner().home(),
            Self::Unpublished(original) => original.owner().home(),
        }
    }
}
