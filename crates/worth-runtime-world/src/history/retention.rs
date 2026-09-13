use std::sync::Arc;

use crate::history::CompositeRuntimeWorldCommit;
use crate::identity::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

use super::catalog::{lock_index, HistoryReachabilityHandle};

/// Closed protection classes keep the history owner from accepting a raw
/// boolean or caller-defined retention authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::history) enum HistoryProtectionClass {
    ProductHead,
    ExplicitObligation,
}

/// One exact installed commit protection. It is move-only and releases its
/// direct protection at most once when the owner of the obligation drops it.
#[must_use = "an exact history protection must remain live while needed"]
pub(in crate::history) struct CompositeHistoryProtectionObligation {
    reachability: HistoryReachabilityHandle,
    identity: CompositeCommitIdentity,
    class: HistoryProtectionClass,
}

impl std::fmt::Debug for CompositeHistoryProtectionObligation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CompositeHistoryProtectionObligation")
            .field("identity", &self.identity)
            .field("class", &self.class)
            .finish_non_exhaustive()
    }
}

impl Drop for CompositeHistoryProtectionObligation {
    fn drop(&mut self) {
        let mut reachability = lock_index(&self.reachability);
        reachability.decrement_direct_protection(&self.identity);
    }
}

impl CompositeHistoryProtectionObligation {
    pub(in crate::history) fn new(
        reachability: Arc<std::sync::Mutex<super::catalog::HistoryReachabilityIndex>>,
        identity: CompositeCommitIdentity,
        class: HistoryProtectionClass,
    ) -> Self {
        Self {
            reachability,
            identity,
            class,
        }
    }
}

/// History-issued proof that one product head keeps its exact installed commit
/// reachable. The generic protection class remains private to History.
#[must_use = "a product head must retain its exact installed commit"]
pub(crate) struct ProductHeadHistoryProtectionObligation {
    protection: CompositeHistoryProtectionObligation,
}

impl std::fmt::Debug for ProductHeadHistoryProtectionObligation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductHeadHistoryProtectionObligation")
            .field("identity", self.commit_identity())
            .finish_non_exhaustive()
    }
}

impl ProductHeadHistoryProtectionObligation {
    pub(in crate::history) fn issued(protection: CompositeHistoryProtectionObligation) -> Self {
        debug_assert_eq!(protection.class, HistoryProtectionClass::ProductHead);
        Self { protection }
    }

    pub(crate) fn commit_identity(&self) -> &CompositeCommitIdentity {
        &self.protection.identity
    }

    pub(crate) fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.commit_identity().owner_identity()
    }

    pub(crate) fn matches_commit(&self, commit: &CompositeRuntimeWorldCommit) -> bool {
        self.commit_identity() == commit.identity()
    }
}

/// History-issued proof that one live commit-bound consumer keeps its exact
/// installed commit reachable. Product-head authority remains a separate
/// capability so callers cannot exchange the two lifecycle roles.
#[must_use = "an explicit commit consumer must retain its exact installed commit"]
pub(crate) struct ExplicitCommitHistoryProtectionObligation {
    protection: CompositeHistoryProtectionObligation,
}

impl std::fmt::Debug for ExplicitCommitHistoryProtectionObligation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExplicitCommitHistoryProtectionObligation")
            .field("identity", self.commit_identity())
            .finish_non_exhaustive()
    }
}

impl ExplicitCommitHistoryProtectionObligation {
    pub(in crate::history) fn issued(protection: CompositeHistoryProtectionObligation) -> Self {
        debug_assert_eq!(protection.class, HistoryProtectionClass::ExplicitObligation);
        Self { protection }
    }

    pub(crate) fn commit_identity(&self) -> &CompositeCommitIdentity {
        &self.protection.identity
    }

    pub(crate) fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.commit_identity().owner_identity()
    }

    pub(crate) fn matches_commit(&self, commit: &CompositeRuntimeWorldCommit) -> bool {
        self.commit_identity() == commit.identity()
    }
}
