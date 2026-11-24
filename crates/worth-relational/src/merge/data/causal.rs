use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::{BranchId, CommitId};
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BranchCausalDot {
    pub branch_id: BranchId,
    pub commit_id: CommitId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalFrontier {
    pub dots: Arc<[BranchCausalDot]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitCausalRelation {
    Before,
    After,
    Equal,
    Concurrent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitCausalMetadata {
    pub observed_frontier: CausalFrontier,
    pub produced_dot: BranchCausalDot,
    pub concurrent_frontier: Arc<[BranchCausalDot]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeCausalEvidenceModel {
    BranchHistoryDerived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeRecordCausalDisposition {
    SourceOnly,
    TargetOnly,
    Equal,
    SourceBeforeTarget,
    SourceAfterTarget,
    Concurrent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeRecordCausalAnnotation {
    pub record: RecordRef,
    pub target_record: Option<RecordRef>,
    pub merge_base_commit_id: CommitId,
    pub source_latest_touch: Option<BranchCausalDot>,
    pub target_latest_touch: Option<BranchCausalDot>,
    pub disposition: MergeRecordCausalDisposition,
    pub evidence_model: MergeCausalEvidenceModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalAnnotationSummary {
    pub classified_record_count: usize,
    pub source_only_count: usize,
    pub target_only_count: usize,
    pub equal_count: usize,
    pub source_before_target_count: usize,
    pub source_after_target_count: usize,
    pub concurrent_count: usize,
    pub evidence_model: MergeCausalEvidenceModel,
    pub annotations: Arc<[MergeRecordCausalAnnotation]>,
}
