use crate::identity::{hash_parts, BasisDigest};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct PlannedWorkPacketDigest(String);

impl PlannedWorkPacketDigest {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(in crate::frontier_planning::testing) fn from_parts(parts: &[String]) -> Self {
        Self(hash_parts(parts))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct BundleResolvedBasisDigest(String);

impl BundleResolvedBasisDigest {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(in crate::frontier_planning::testing) fn from_basis_digest(digest: &BasisDigest) -> Self {
        Self(digest.as_str().to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FrontierPostureDigest(String);

impl FrontierPostureDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(in crate::frontier_planning::testing) fn from_parts(parts: &[String]) -> Self {
        Self(hash_parts(parts))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FrontierSurfaceDigest(String);

impl FrontierSurfaceDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_label(label: &str) -> Self {
        Self(hash_parts(&[format!("frontier_surface:{label}")]))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum PacketEquivalenceContract {
    CollectionDigestAndBasis,
    BoundedTraversalDigestAndBasis,
    LiveDescriptorAndProgressBasis,
}

impl PacketEquivalenceContract {
    pub(in crate::frontier_planning::testing) fn as_str(&self) -> &'static str {
        match self {
            Self::CollectionDigestAndBasis => "collection_digest_and_basis",
            Self::BoundedTraversalDigestAndBasis => "bounded_traversal_digest_and_basis",
            Self::LiveDescriptorAndProgressBasis => "live_descriptor_and_progress_basis",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum PacketMergeContract {
    OrderedCollectionResultBoundary,
    BoundedMaterializationResultBoundary,
    LiveDetailResultBoundary,
    LiveOrderedCollectionResultBoundary,
    LiveBoundedMaterializationResultBoundary,
}

impl PacketMergeContract {
    fn as_str(&self) -> &'static str {
        match self {
            Self::OrderedCollectionResultBoundary => "ordered_collection_result_boundary",
            Self::BoundedMaterializationResultBoundary => "bounded_materialization_result_boundary",
            Self::LiveDetailResultBoundary => "live_detail_result_boundary",
            Self::LiveOrderedCollectionResultBoundary => "live_ordered_collection_result_boundary",
            Self::LiveBoundedMaterializationResultBoundary => {
                "live_bounded_materialization_result_boundary"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PacketMergeBoundary {
    contract: PacketMergeContract,
    digest: FrontierPostureDigest,
}

impl PacketMergeBoundary {
    pub(crate) fn contract(&self) -> &PacketMergeContract {
        &self.contract
    }

    pub(crate) fn digest(&self) -> &FrontierPostureDigest {
        &self.digest
    }

    pub(in crate::frontier_planning::testing) fn new(
        contract: PacketMergeContract,
        scope_summary: &str,
        basis: &BundleResolvedBasisDigest,
    ) -> Self {
        Self {
            digest: FrontierPostureDigest::from_parts(&[
                format!("merge_contract:{}", contract.as_str()),
                format!("scope:{scope_summary}"),
                format!("basis:{}", basis.as_str()),
            ]),
            contract,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FrontierPredictionDriftOutcome {
    WithinBudget,
    SerialFallbackRequired,
    DeniedByDrift,
}

impl FrontierPredictionDriftOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WithinBudget => "within_budget",
            Self::SerialFallbackRequired => "serial_fallback_required",
            Self::DeniedByDrift => "denied_by_drift",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FrontierBreadthPrediction(usize);

impl FrontierBreadthPrediction {
    pub fn value(&self) -> usize {
        self.0
    }

    pub(in crate::frontier_planning::testing) fn new(value: usize) -> Self {
        Self(value.max(1))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FrontierComplexityContract(&'static str);

impl FrontierComplexityContract {
    pub fn as_str(&self) -> &str {
        self.0
    }

    pub(in crate::frontier_planning::testing) fn ordered_collection() -> Self {
        Self("frontier_ordered_collection")
    }

    pub(in crate::frontier_planning::testing) fn bounded_materialization() -> Self {
        Self("frontier_bounded_materialization")
    }

    pub(in crate::frontier_planning::testing) fn live_detail() -> Self {
        Self("frontier_live_detail")
    }

    pub(in crate::frontier_planning::testing) fn live_ordered_collection() -> Self {
        Self("frontier_live_ordered_collection")
    }

    pub(in crate::frontier_planning::testing) fn live_bounded_materialization() -> Self {
        Self("frontier_live_bounded_materialization")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FrontierPerformanceStatus {
    Verified,
    Debt,
}

impl FrontierPerformanceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Debt => "debt",
        }
    }
}
