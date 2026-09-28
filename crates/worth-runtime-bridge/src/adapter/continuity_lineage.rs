use sha2::{Digest, Sha256};

use crate::identity::{
    BridgeIdentity, HistoricalResolvedLineageIdentityTag, HistoricalResolvedRecordIdentityTag,
};

use super::*;

pub type BridgeHistoricalResolvedLineageIdentity =
    BridgeIdentity<HistoricalResolvedLineageIdentityTag>;
pub type BridgeHistoricalResolvedRecordIdentity =
    BridgeIdentity<HistoricalResolvedRecordIdentityTag>;

pub trait ContinuityLineageSource: Send + Sync + 'static {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeHistoricalLineageTopology {
    NoAuthoritativeSuccessor,
    UnsupportedWithoutSuccessor,
    SingleSuccessor,
    MergeLikeSuccessor,
    SplitSuccessors,
    AmbiguousSuccessor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalLineageRequest {
    authority_basis: continuity::BridgeContinuityAuthorityBasis,
    prior_slice: continuity::PriorSubscriptionSlice,
}

impl BridgeHistoricalLineageRequest {
    pub fn new(
        authority_basis: continuity::BridgeContinuityAuthorityBasis,
        prior_slice: continuity::PriorSubscriptionSlice,
    ) -> Self {
        Self {
            authority_basis,
            prior_slice,
        }
    }

    pub fn authority_basis(&self) -> &continuity::BridgeContinuityAuthorityBasis {
        &self.authority_basis
    }

    pub fn prior_slice(&self) -> &continuity::PriorSubscriptionSlice {
        &self.prior_slice
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalLineageAuthority {
    authority_basis: continuity::BridgeContinuityAuthorityBasis,
    canonical_resolved_lineage_identities: Arc<[BridgeHistoricalResolvedLineageIdentity]>,
    canonical_resolved_record_identities: Arc<[BridgeHistoricalResolvedRecordIdentity]>,
    traversed_event_ids: Arc<[u64]>,
    topology: BridgeHistoricalLineageTopology,
    lineage_digest: Arc<str>,
}

impl BridgeHistoricalLineageAuthority {
    /// The answer a `ContinuityLineageSource` gives for one lineage request.
    ///
    /// This is public because the trait is: any source, including a
    /// certification fake inside Query's public acceptance suite, must be able
    /// to build its answer. The authority basis comes from the request being
    /// answered, and the identities and event ids must be canonical and free of
    /// duplicates, or construction fails.
    pub fn try_new(
        authority_basis: continuity::BridgeContinuityAuthorityBasis,
        canonical_resolved_lineage_identities: Vec<BridgeHistoricalResolvedLineageIdentity>,
        canonical_resolved_record_identities: Vec<BridgeHistoricalResolvedRecordIdentity>,
        traversed_event_ids: Vec<u64>,
    ) -> Result<Self, BridgeLineageSourceError> {
        let mut sorted_lineage_identities = canonical_resolved_lineage_identities.clone();
        sorted_lineage_identities.sort_unstable();
        sorted_lineage_identities.dedup();
        if sorted_lineage_identities.len() != canonical_resolved_lineage_identities.len() {
            return Err(BridgeLineageSourceError::new(
                BridgeLineageSourceErrorKind::DuplicateResolvedLineageIdentities,
                "bridge historical lineage authority contained duplicate resolved lineage identities",
            ));
        }
        if sorted_lineage_identities != canonical_resolved_lineage_identities {
            return Err(BridgeLineageSourceError::new(
                BridgeLineageSourceErrorKind::NonCanonicalResolvedLineageIdentities,
                "bridge historical lineage authority must emit resolved lineage identities in canonical order",
            ));
        }

        let mut sorted_record_identities = canonical_resolved_record_identities.clone();
        sorted_record_identities.sort_unstable();
        sorted_record_identities.dedup();
        if sorted_record_identities.len() != canonical_resolved_record_identities.len() {
            return Err(BridgeLineageSourceError::new(
                BridgeLineageSourceErrorKind::DuplicateResolvedRecordIdentities,
                "bridge historical lineage authority contained duplicate resolved record identities",
            ));
        }
        if sorted_record_identities != canonical_resolved_record_identities {
            return Err(BridgeLineageSourceError::new(
                BridgeLineageSourceErrorKind::NonCanonicalResolvedRecordIdentities,
                "bridge historical lineage authority must emit resolved record identities in canonical order",
            ));
        }

        let mut sorted_event_ids = traversed_event_ids.clone();
        sorted_event_ids.sort_unstable();
        sorted_event_ids.dedup();
        if sorted_event_ids.len() != traversed_event_ids.len() {
            return Err(BridgeLineageSourceError::new(
                BridgeLineageSourceErrorKind::DuplicateTraversedEventIds,
                "bridge historical lineage authority contained duplicate traversed event ids",
            ));
        }
        if sorted_event_ids != traversed_event_ids {
            return Err(BridgeLineageSourceError::new(
                BridgeLineageSourceErrorKind::NonCanonicalTraversedEventIds,
                "bridge historical lineage authority must emit traversed event ids in canonical order",
            ));
        }

        let topology = match (
            canonical_resolved_lineage_identities.len(),
            canonical_resolved_record_identities.len(),
        ) {
            (0, 0) => BridgeHistoricalLineageTopology::NoAuthoritativeSuccessor,
            (_, 0) => BridgeHistoricalLineageTopology::UnsupportedWithoutSuccessor,
            (_, 1) if canonical_resolved_lineage_identities.len() > 1 => {
                BridgeHistoricalLineageTopology::MergeLikeSuccessor
            }
            (_, 1) => BridgeHistoricalLineageTopology::SingleSuccessor,
            (lineage_count, record_count) if lineage_count == record_count => {
                BridgeHistoricalLineageTopology::SplitSuccessors
            }
            _ => BridgeHistoricalLineageTopology::AmbiguousSuccessor,
        };

        let canonical_basis = format!(
            "historical-lineage-authority|authority={}|branch={}|snapshot={}|lineage-count={}|lineages={}|record-count={}|records={}|event-count={}|events={}",
            authority_basis.digest(),
            authority_basis.branch_identity().as_str(),
            authority_basis.snapshot_identity().as_str(),
            canonical_resolved_lineage_identities.len(),
            canonical_resolved_lineage_identities
                .iter()
                .map(BridgeHistoricalResolvedLineageIdentity::as_str)
                .collect::<Vec<_>>()
                .join(","),
            canonical_resolved_record_identities.len(),
            canonical_resolved_record_identities
                .iter()
                .map(BridgeHistoricalResolvedRecordIdentity::as_str)
                .collect::<Vec<_>>()
                .join(","),
            traversed_event_ids.len(),
            traversed_event_ids
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            authority_basis,
            canonical_resolved_lineage_identities: Arc::from(canonical_resolved_lineage_identities),
            canonical_resolved_record_identities: Arc::from(canonical_resolved_record_identities),
            traversed_event_ids: Arc::from(traversed_event_ids),
            topology,
            lineage_digest: Arc::from(format!("historical-lineage-authority:sha256:{digest:x}")),
        })
    }

    pub fn authority_basis(&self) -> &continuity::BridgeContinuityAuthorityBasis {
        &self.authority_basis
    }

    pub fn branch_identity(&self) -> &input::envelope::TruthBranchIdentity {
        self.authority_basis.branch_identity()
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        self.authority_basis.snapshot_identity()
    }

    pub fn canonical_resolved_lineage_identities(
        &self,
    ) -> &[BridgeHistoricalResolvedLineageIdentity] {
        &self.canonical_resolved_lineage_identities
    }

    pub fn canonical_resolved_record_identities(
        &self,
    ) -> &[BridgeHistoricalResolvedRecordIdentity] {
        &self.canonical_resolved_record_identities
    }

    pub fn traversed_event_ids(&self) -> &[u64] {
        &self.traversed_event_ids
    }

    pub fn topology(&self) -> BridgeHistoricalLineageTopology {
        self.topology
    }

    pub fn lineage_digest(&self) -> &str {
        self.lineage_digest.as_ref()
    }
}
