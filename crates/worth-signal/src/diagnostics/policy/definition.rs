use serde::{Deserialize, Serialize};

use super::materialization::ArtifactRetentionPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayDetailPolicy {
    Minimal,
    Standard,
    Forensic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRetentionPolicy {
    Minimal,
    Development,
    Forensic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotRestoreLineageMode {
    CompactGlobal,
    PerNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HistoryLimit(usize);

impl HistoryLimit {
    pub fn new(value: usize) -> Self {
        Self(value.max(1))
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn max(self, other: usize) -> usize {
        self.0.max(other)
    }
}

impl std::fmt::Display for HistoryLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Mul<usize> for HistoryLimit {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        self.0 * rhs
    }
}

impl PartialEq<usize> for HistoryLimit {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<usize> for HistoryLimit {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<HistoryLimit> for usize {
    fn eq(&self, other: &HistoryLimit) -> bool {
        *self == other.0
    }
}

impl PartialOrd<HistoryLimit> for usize {
    fn partial_cmp(&self, other: &HistoryLimit) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DetailLimit(usize);

impl DetailLimit {
    pub fn new(value: usize) -> Self {
        Self(value.max(1))
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn max(self, other: usize) -> usize {
        self.0.max(other)
    }
}

impl std::fmt::Display for DetailLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<usize> for DetailLimit {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<usize> for DetailLimit {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<DetailLimit> for usize {
    fn eq(&self, other: &DetailLimit) -> bool {
        *self == other.0
    }
}

impl PartialOrd<DetailLimit> for usize {
    fn partial_cmp(&self, other: &DetailLimit) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionBudget {
    pub history_limit: HistoryLimit,
    pub detail_limit: DetailLimit,
    pub retain_history_details: bool,
    pub retain_flow_explanation: bool,
    pub retain_latest_failure_context: bool,
    pub retain_stage_details: bool,
    pub capture_forensic_failure_context: bool,
    pub explanation_retention: ArtifactRetentionPolicy,
    pub provenance_retention: ArtifactRetentionPolicy,
    pub replay_detail: ReplayDetailPolicy,
    pub semantic_detail: SemanticRetentionPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructionBudget {
    pub allow_explanation_reconstruction: bool,
    pub allow_provenance_reconstruction: bool,
    pub allow_replay_reconstruction: bool,
    pub allow_certification_materialization: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FrontierTracingPolicy {
    #[default]
    SummaryOnly,
    RetainWaveRecords,
    FullForensic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FrontierPropagationPolicy {
    #[default]
    CanonicalFrontier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FrontierCyclePolicy {
    #[default]
    ReachableCycleCheck,
}
