use crate::identity::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

use super::catalog::CompositeHistoryCatalogDenial;

/// Explicit maintenance input. Product heads and retained obligations are
/// represented by live catalog-owned protection obligations, not copied
/// ancestry supplied with this request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeHistoryReclamationRequest {
    owner: RuntimeWorldOwnerIdentity,
    candidate_commits: Vec<CompositeCommitIdentity>,
    maximum_reclaims: usize,
}

impl CompositeHistoryReclamationRequest {
    pub fn new(
        owner: RuntimeWorldOwnerIdentity,
        candidate_commits: Vec<CompositeCommitIdentity>,
        maximum_reclaims: usize,
    ) -> Self {
        Self {
            owner,
            candidate_commits,
            maximum_reclaims,
        }
    }

    pub const fn owner(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub fn candidate_commits(&self) -> &[CompositeCommitIdentity] {
        &self.candidate_commits
    }

    pub const fn maximum_reclaims(&self) -> usize {
        self.maximum_reclaims
    }
}

/// Reclamation is an observation of a bounded maintenance batch, not a
/// promise that every requested candidate can be removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryReclamationOutcome {
    maximum_reclaims: usize,
    examined: usize,
    skipped_protected: usize,
    skipped_with_descendant_dependencies: usize,
    reclaimed: Vec<CompositeCommitIdentity>,
    metadata_bytes_reclaimed: usize,
}

impl HistoryReclamationOutcome {
    pub(crate) fn new(maximum_reclaims: usize) -> Self {
        Self {
            maximum_reclaims,
            examined: 0,
            skipped_protected: 0,
            skipped_with_descendant_dependencies: 0,
            reclaimed: Vec::new(),
            metadata_bytes_reclaimed: 0,
        }
    }

    pub(crate) fn examined_one(&mut self) {
        self.examined += 1;
    }

    pub(crate) fn record_skipped_protected(&mut self) {
        self.skipped_protected += 1;
    }

    pub(crate) fn record_skipped_with_descendant_dependencies(&mut self) {
        self.skipped_with_descendant_dependencies += 1;
    }

    pub(crate) fn reclaimed_one(
        &mut self,
        identity: CompositeCommitIdentity,
        metadata_bytes: usize,
    ) {
        debug_assert!(self.reclaimed.len() < self.maximum_reclaims);
        self.metadata_bytes_reclaimed = self
            .metadata_bytes_reclaimed
            .checked_add(metadata_bytes)
            .expect("a bounded reclamation outcome fits addressable memory");
        self.reclaimed.push(identity);
    }

    pub const fn maximum_reclaims(&self) -> usize {
        self.maximum_reclaims
    }

    pub const fn examined(&self) -> usize {
        self.examined
    }

    pub const fn skipped_protected(&self) -> usize {
        self.skipped_protected
    }

    pub const fn skipped_with_descendant_dependencies(&self) -> usize {
        self.skipped_with_descendant_dependencies
    }

    pub fn reclaimed_commits(&self) -> &[CompositeCommitIdentity] {
        &self.reclaimed
    }

    pub const fn metadata_bytes_reclaimed(&self) -> usize {
        self.metadata_bytes_reclaimed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryReclamationDenial {
    OwnerUnavailable(crate::lifecycle::RuntimeWorldOwnerUnavailable),
    Catalog(CompositeHistoryCatalogDenial),
    ForeignCandidate {
        expected: RuntimeWorldOwnerIdentity,
        actual: RuntimeWorldOwnerIdentity,
    },
    DuplicateCandidate(CompositeCommitIdentity),
    UnknownCandidate(CompositeCommitIdentity),
}
