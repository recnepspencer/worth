use std::mem::size_of;
use std::sync::{Arc, Mutex, OnceLock};

use crate::identity::CompositeCommitIdentity;

use super::super::super::CompositeCommitParent;
use super::super::reachability::HistoryReachabilityRecord;
use super::super::{
    CompositeHistoryCatalogEntry, CompositeRuntimeWorldCommit, HistoryReservationMetadata,
};

/// Test-local logical allocation model. It intentionally repeats the named
/// layout charges instead of calling any production metadata function.
pub(super) struct AllocationOracle;

impl AllocationOracle {
    pub(super) fn publication_resident(
        commit: &CompositeRuntimeWorldCommit,
        branch_name: &str,
    ) -> usize {
        sum([
            Self::installed_resident(commit),
            size_of::<crate::history::CanonicalPublicationEnvelope>(),
            size_of::<Arc<crate::history::CanonicalPublicationEnvelope>>(),
            size_of::<Arc<str>>(),
            branch_name.len(),
        ])
    }

    pub(super) fn installed_resident(_commit: &CompositeRuntimeWorldCommit) -> usize {
        sum([
            size_of::<CompositeRuntimeWorldCommit>(),
            size_of::<Arc<CompositeRuntimeWorldCommit>>(),
            size_of::<CompositeCommitIdentity>(),
            size_of::<Arc<OnceLock<CompositeHistoryCatalogEntry>>>(),
            size_of::<CompositeCommitIdentity>(),
            size_of::<Arc<Mutex<Option<HistoryReachabilityRecord>>>>(),
            size_of::<OnceLock<CompositeHistoryCatalogEntry>>(),
            size_of::<Mutex<Option<HistoryReachabilityRecord>>>(),
            4 * size_of::<usize>(),
        ])
    }

    pub(super) fn reservation_resident(commit: &CompositeRuntimeWorldCommit) -> usize {
        let held_parent_identity = match commit.parent() {
            CompositeCommitParent::Root => 0,
            CompositeCommitParent::Ordinary(_) => size_of::<CompositeCommitIdentity>(),
        };
        sum([
            size_of::<CompositeCommitIdentity>(),
            size_of::<HistoryReservationMetadata>(),
            size_of::<CompositeCommitIdentity>(),
            held_parent_identity,
            size_of::<Arc<Mutex<super::super::CompositeHistoryCatalogState>>>(),
            2 * size_of::<usize>(),
        ])
    }

    pub(super) fn reservation_plus_installation(commit: &CompositeRuntimeWorldCommit) -> usize {
        sum([
            Self::reservation_resident(commit),
            Self::installed_resident(commit),
        ])
    }
}

fn sum<const N: usize>(parts: [usize; N]) -> usize {
    parts
        .into_iter()
        .try_fold(0usize, |total, part| total.checked_add(part))
        .expect("the independent test allocation model fits usize")
}
