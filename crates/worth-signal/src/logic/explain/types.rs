mod retained_charge;

use serde::{Deserialize, Serialize};

use std::fmt;

use crate::data::aspect::{Aspect, AspectMask};
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::handle::NodeId;
use crate::data::node::{ContextRequirement, EvaluationCondition, NodeState};
use crate::data::output::{
    ChangedRegion, MemoizedResultOrigin, OutputChange, OutputIdentity, PartitionSubscription,
};
use crate::data::reuse::{ReuseBasis, ReuseCertificationRecord};
use crate::data::trace::{CausalityMetadata, HistoricalArtifactRecord, TraceSummary};
use crate::diagnostics::policy::DiagnosticsAvailability;

mod presentation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeaningfulChangeReason {
    ExactDifference,
    Tolerance { epsilon: u64 },
    OutputIdentity,
    CustomComparator { key: String },
    InstalledComparator,
    InheritedComparator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionDecision {
    Deferred,
    RevertedClean,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpstreamCause {
    Changed {
        source: NodeId,
        aspect: Aspect,
        subscription: Option<PartitionSubscription>,
        cached_version: u64,
        current_version: u64,
        comparator: VersionComparatorPolicy,
        reason: MeaningfulChangeReason,
    },
    SkippedByComparator {
        source: NodeId,
        aspect: Aspect,
        subscription: Option<PartitionSubscription>,
        cached_version: u64,
        current_version: u64,
        comparator: VersionComparatorPolicy,
        reason: MeaningfulChangeReason,
    },
    ConditionDeferred {
        source: NodeId,
        aspect: Aspect,
        subscription: Option<PartitionSubscription>,
        cached_version: u64,
        current_version: u64,
        condition: EvaluationCondition,
        decision: ConditionDecision,
    },
    Clean {
        source: NodeId,
        aspect: Aspect,
        subscription: Option<PartitionSubscription>,
        cached_version: u64,
        current_version: u64,
    },
    MissingSnapshot {
        source: NodeId,
        aspect: Aspect,
        subscription: Option<PartitionSubscription>,
        current_version: Option<u64>,
    },
    DependencyRemoved {
        source: NodeId,
        aspect: Aspect,
        subscription: Option<PartitionSubscription>,
        cached_version: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CausalDisposition {
    Semantic,
    Suppressed,
    Ignored,
    Conservative,
    Topology,
    Lifecycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeProvenanceKind {
    None,
    Direct,
    Translated,
    Discarded,
    InsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CausalLinkKind {
    Changed,
    SkippedByComparator,
    ConditionDeferred {
        condition: EvaluationCondition,
        decision: ConditionDecision,
    },
    ScopeUntouched,
    Clean,
    MissingSnapshot,
    DependencyAdded,
    DependencyRemoved,
}

impl fmt::Display for CausalLinkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Changed => write!(f, "Changed"),
            Self::SkippedByComparator => write!(f, "SkippedByComparator"),
            Self::ConditionDeferred {
                condition,
                decision,
            } => write!(f, "ConditionDeferred::{condition:?}/{decision:?}"),
            Self::ScopeUntouched => write!(f, "ScopeUntouched"),
            Self::Clean => write!(f, "Clean"),
            Self::MissingSnapshot => write!(f, "MissingSnapshot"),
            Self::DependencyAdded => write!(f, "DependencyAdded"),
            Self::DependencyRemoved => write!(f, "DependencyRemoved"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeProvenance {
    pub source_scope: Option<PartitionSubscription>,
    pub validation_scope: Option<PartitionSubscription>,
    pub kind: ScopeProvenanceKind,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalLink {
    pub source: Option<NodeId>,
    pub aspect: Option<Aspect>,
    pub disposition: CausalDisposition,
    pub kind: CausalLinkKind,
    pub scope: ScopeProvenance,
    pub cached_version: Option<u64>,
    pub current_version: Option<u64>,
    pub comparator: Option<VersionComparatorPolicy>,
    pub reason: Option<MeaningfulChangeReason>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewiringDependency {
    pub source: NodeId,
    pub aspect: Aspect,
    pub subscription: Option<PartitionSubscription>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewiringSummary {
    pub added: Vec<RewiringDependency>,
    pub removed: Vec<RewiringDependency>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeExplanation {
    pub node: NodeId,
    #[serde(default)]
    pub materialization_mode: DiagnosticsAvailability,
    pub state: NodeState,
    pub dirty_aspects: AspectMask,
    pub contract_reads: AspectMask,
    pub contract_produces: AspectMask,
    pub contract_partition_scope: Option<Vec<PartitionSubscription>>,
    pub required_context: ContextRequirement,
    pub condition: EvaluationCondition,
    pub historical_artifact_record: Option<HistoricalArtifactRecord>,
    pub execution_record_id: Option<u64>,
    pub semantic_segment_id: Option<u64>,
    pub output_identity: Option<OutputIdentity>,
    pub output_change: Option<OutputChange>,
    pub changed_regions: Vec<ChangedRegion>,
    pub propagation_suppressed: bool,
    pub memoized_origin: Option<MemoizedResultOrigin>,
    pub reuse_basis: Option<ReuseBasis>,
    pub reuse_origin: Option<crate::data::reuse::ReuseOrigin>,
    pub reuse_certification: Option<ReuseCertificationRecord>,
    pub upstream: Vec<UpstreamCause>,
    #[serde(default)]
    pub causal_links: Vec<CausalLink>,
    #[serde(default)]
    pub rewiring: Option<RewiringSummary>,
    pub causality: Option<CausalityMetadata>,
}

impl NodeExplanation {
    pub fn materialized_trace_summary(&self) -> Option<TraceSummary> {
        self.historical_artifact_record
            .as_ref()
            .map(TraceSummary::from_record)
    }
}

pub(super) fn reason_for_policy(
    policy: &VersionComparatorPolicy,
    explicit: bool,
) -> MeaningfulChangeReason {
    match policy {
        VersionComparatorPolicy::Exact => {
            if explicit {
                MeaningfulChangeReason::ExactDifference
            } else {
                MeaningfulChangeReason::InheritedComparator
            }
        }
        VersionComparatorPolicy::Tolerance { epsilon } => {
            MeaningfulChangeReason::Tolerance { epsilon: *epsilon }
        }
        VersionComparatorPolicy::OutputIdentity => MeaningfulChangeReason::OutputIdentity,
        VersionComparatorPolicy::Custom { key } => {
            MeaningfulChangeReason::CustomComparator { key: key.clone() }
        }
        VersionComparatorPolicy::Installed { .. } => MeaningfulChangeReason::InstalledComparator,
    }
}
