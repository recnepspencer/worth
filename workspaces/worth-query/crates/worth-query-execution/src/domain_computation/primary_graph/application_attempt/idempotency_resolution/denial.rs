//! Why an idempotency key could not be resolved.

use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Provider;
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

/// Why an idempotency key could not be resolved. The resolution is a read, so
/// nothing took effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationIdempotencyResolutionDenialKind {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
    /// The admission's current authority no longer holds, or inspecting the key was
    /// not authorized, for the named reason. The authorization denial has the
    /// contributing causes.
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    /// The admission belongs to another runtime or schema binding, or has no product
    /// to resolve against.
    ForeignAdmission,
    /// The provider holds its maximum number of active snapshots.
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    /// The provider has no retention capacity for the read.
    RetentionCapacityExhausted,
    /// The provider has run out of retention identities.
    RetentionIdentityExhausted,
    /// The provider has run out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The provider could not answer, the product was unpublished, or the
    /// recorded receipt could not be projected.
    ProviderUnavailable,
    /// The key is recorded with the same intent at `commit`, but this runtime
    /// no longer holds that commit's receipt: it was committed before a
    /// restore or reopen, or its product was retired. The commit took effect;
    /// retrying the same request cannot commit it again.
    CommittedReceiptNotRetained {
        commit: worth_relational::facade::history::CommitId,
    },
    /// The key's recorded intent was written by an earlier encoding whose
    /// operation part cannot be checked against this request. Neither the same
    /// intent nor drift is proven.
    RecordedIntentUnverifiable,
    /// The key's commit left the declared idempotency window.
    IdempotencyWindowExpired,
}

/// A refusal to resolve an idempotency key. Nothing took effect.
///
/// Match on [`kind`](Self::kind); [`authorization`](Self::authorization) is set
/// only for an authorization refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationIdempotencyResolutionDenial {
    kind: WorthQueryApplicationIdempotencyResolutionDenialKind,
    authorization: Option<WorthQueryOperationAuthorizationDenial>,
}

impl WorthQueryApplicationIdempotencyResolutionDenial {
    /// The key's earlier commit is proven, but its live receipt is unavailable.
    /// Other denials carry no historical commit observation.
    pub const fn historical_commit(&self) -> Option<super::WorthQueryHistoricalApplicationCommit> {
        match self.kind {
            WorthQueryApplicationIdempotencyResolutionDenialKind::CommittedReceiptNotRetained {
                commit,
            } => Some(super::WorthQueryHistoricalApplicationCommit::observed(
                commit,
            )),
            _ => None,
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationIdempotencyResolutionDenialKind {
        self.kind
    }

    pub const fn authorization(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization.as_ref()
    }

    pub(super) fn from_authorization(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization(
                denial.kind(),
            ),
            authorization: Some(denial),
        }
    }

    pub(super) const fn foreign_admission() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission,
            authorization: None,
        }
    }

    pub(super) const fn provider_unavailable() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable,
            authorization: None,
        }
    }

    pub(super) const fn branch_coordination_capacity_exhausted() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionCapacityExhausted,
            authorization: None,
        }
    }

    pub(super) fn from_provider(denial: Provider) -> Self {
        let kind = match denial {
            Provider::Handle(denial) => return denial.into(),
            Provider::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => WorthQueryApplicationIdempotencyResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Provider::RetentionCapacityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionCapacityExhausted
            }
            Provider::RetentionIdentityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionIdentityExhausted
            }
            Provider::SnapshotIdentityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::SnapshotIdentityExhausted
            }
            Provider::Unavailable => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
            }
            Provider::CommittedReceiptNotRetained { commit } => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::CommittedReceiptNotRetained {
                    commit,
                }
            }
            Provider::RecordedIntentUnverifiable => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RecordedIntentUnverifiable
            }
            Provider::WindowExpired => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::IdempotencyWindowExpired
            }
        };
        Self {
            kind,
            authorization: None,
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationIdempotencyResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application idempotency resolution denied: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for WorthQueryApplicationIdempotencyResolutionDenial {}

impl From<crate::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryApplicationIdempotencyResolutionDenial
{
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::Handle(denial),
            authorization: None,
        }
    }
}
