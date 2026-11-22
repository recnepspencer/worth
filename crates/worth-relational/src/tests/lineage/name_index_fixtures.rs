//! Shared helpers for the derived name index courts.

use crate::facade::history::{BranchId, CommitId};
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameKey,
    PersistentNameResolution, PersistentNameResolutionDenial, PersistentNameResolutionMetrics,
    PersistentNameSelection, PersistentNameTarget,
};

use super::name_world::{side_of, Name};

pub(super) type Answer = Result<PersistentNameResolution, PersistentNameResolutionDenial>;

pub(super) fn main_branch() -> BranchId {
    BranchId("main".to_owned())
}

pub(super) fn rebuild(log: &[PersistentNameCommittedBatch]) -> PersistentNameBranchIndex {
    PersistentNameBranchIndex::rebuild_from_log(main_branch(), None, log)
        .expect("canonical committed log rebuilds the index")
}

pub(super) fn resolve(
    index: &PersistentNameBranchIndex,
    held: PersistentNameKey,
    basis: u64,
    selection: &PersistentNameSelection,
) -> Answer {
    index.resolve(held, CommitId(basis), selection).result
}

pub(super) fn metrics(
    index: &PersistentNameBranchIndex,
    held: PersistentNameKey,
    basis: u64,
) -> PersistentNameResolutionMetrics {
    index
        .resolve(held, CommitId(basis), &PersistentNameSelection::Exact)
        .metrics
}

pub(super) fn exact() -> PersistentNameSelection {
    PersistentNameSelection::Exact
}

pub(super) fn all() -> PersistentNameSelection {
    PersistentNameSelection::AllDescendants
}

pub(super) fn bounded(carriers: &[Name]) -> PersistentNameSelection {
    PersistentNameSelection::BoundedByCarriers {
        carriers: carriers.iter().copied().collect(),
    }
}

pub(super) fn side(label: &str) -> PersistentNameSelection {
    PersistentNameSelection::KeepSide {
        discriminator: side_of(label),
    }
}

/// The key an `Exact` answer resolved to.
pub(super) fn exact_key(answer: &Answer) -> PersistentNameKey {
    match answer {
        Ok(PersistentNameResolution::Exact(target)) => target.key(),
        other => panic!("expected an exact answer, found {other:?}"),
    }
}

pub(super) fn plural_keys(answer: &Answer) -> Vec<PersistentNameKey> {
    match answer {
        Ok(PersistentNameResolution::Plural(targets)) => {
            targets.iter().map(PersistentNameTarget::key).collect()
        }
        other => panic!("expected a plural answer, found {other:?}"),
    }
}

pub(super) fn ambiguous(mut candidates: Vec<PersistentNameKey>) -> Answer {
    candidates.sort_unstable();
    Err(PersistentNameResolutionDenial::Ambiguous { candidates })
}

pub(super) fn no_match(mut live_candidates: Vec<PersistentNameKey>) -> Answer {
    live_candidates.sort_unstable();
    Err(PersistentNameResolutionDenial::NoMatchingCandidate { live_candidates })
}

pub(super) fn retired(at: u64) -> Answer {
    Ok(PersistentNameResolution::Retired {
        at_commit: CommitId(at),
    })
}

pub(super) fn unknown(name: PersistentNameKey) -> Answer {
    Err(PersistentNameResolutionDenial::UnknownName { name })
}

pub(super) fn probes(
    index_probes: usize,
    event_visits: usize,
    edge_visits: usize,
) -> PersistentNameResolutionMetrics {
    PersistentNameResolutionMetrics {
        index_probes,
        edge_visits,
        event_visits,
    }
}
