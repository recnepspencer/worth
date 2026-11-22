//! Recorded logical cleanup and the actual live owner's resource release.
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductUnpublishedOwnerEffectsIdentity,
    ManagedCleanupObligationIdentity,
};
use worth_query_execution_foundation::facade::domain_computation::managed_run_vocabulary::{
    WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunTerminalKind,
    WorthQueryMutationCleanupFailureKind,
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
    /// Recorded once, before service, when the incarnation holding this
    /// cleanup ended. The failure and original stay as recorded.
    pub cold_release: Option<super::QueryColdReleaseTerminal>,
}
impl QueryManagedCleanupObligation {
    /// The live owner recorded its release, or the cleanup's incarnation ended.
    pub const fn released(&self) -> bool {
        self.release.is_some() || self.cold_release.is_some()
    }
    /// Inconsistent business evidence is not a resource failure: no release,
    /// live or cold, may repair it.
    pub(super) const fn evidence_inconsistent(&self) -> bool {
        matches!(
            self.failure,
            WorthQueryMutationCleanupFailureKind::AffinityInconsistent
                | WorthQueryMutationCleanupFailureKind::CompletionInconsistent
        )
    }
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
