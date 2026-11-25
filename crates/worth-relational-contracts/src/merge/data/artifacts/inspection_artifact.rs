use serde::{Deserialize, Serialize};

use crate::merge::data::conflicts::MergeConflictClass;
use crate::merge::data::inspection_digest::{
    merge_inspection_artifact_digest, merge_inspection_lowered_plan_digest,
    merge_inspection_row_digest,
};
use crate::merge::data::policy::{
    LoweredMergeBlockedReason, LoweredMergePlanSummary, LoweredMergeRejectedReason,
    LoweredRecordDecision, LoweredRecordDecisionKind, MergeExecutionReadiness,
    MergeResolutionClass,
};
use crate::merge::data::requests::normalized::NormalizedRelationalMergeRequest;
use crate::transactions::data::primitives::RecordRef;

use super::MergePlanningArtifactCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeInspectionAdmission {
    ExecutionAdmissible,
    ExecutionDenied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalMergeInspectionInput {
    request: NormalizedRelationalMergeRequest,
    lowered_plan: LoweredMergePlanSummary,
}

impl RelationalMergeInspectionInput {
    fn from_planning_artifact(artifact: &MergePlanningArtifactCore) -> Self {
        Self {
            request: artifact.request.clone(),
            lowered_plan: artifact.lowered_plan.clone(),
        }
    }

    pub fn request(&self) -> &NormalizedRelationalMergeRequest {
        &self.request
    }

    pub fn lowered_plan(&self) -> &LoweredMergePlanSummary {
        &self.lowered_plan
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalMergeInspectionRow {
    record: RecordRef,
    target_record: Option<RecordRef>,
    classification: MergeConflictClass,
    resolution_class: MergeResolutionClass,
    readiness: MergeExecutionReadiness,
    decision_kind: LoweredRecordDecisionKind,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    rejected_reason: Option<LoweredMergeRejectedReason>,
    admission: RelationalMergeInspectionAdmission,
    row_digest: String,
}

impl RelationalMergeInspectionRow {
    pub fn record(&self) -> &RecordRef {
        &self.record
    }

    pub fn target_record(&self) -> Option<&RecordRef> {
        self.target_record.as_ref()
    }

    pub fn classification(&self) -> &MergeConflictClass {
        &self.classification
    }

    pub fn resolution_class(&self) -> &MergeResolutionClass {
        &self.resolution_class
    }

    pub fn readiness(&self) -> &MergeExecutionReadiness {
        &self.readiness
    }

    pub fn decision_kind(&self) -> LoweredRecordDecisionKind {
        self.decision_kind
    }

    pub fn blocked_reason(&self) -> Option<LoweredMergeBlockedReason> {
        self.blocked_reason
    }

    pub fn rejected_reason(&self) -> Option<LoweredMergeRejectedReason> {
        self.rejected_reason
    }

    pub fn admission(&self) -> RelationalMergeInspectionAdmission {
        self.admission
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalMergeInspectionArtifact {
    request: NormalizedRelationalMergeRequest,
    lowered_plan_digest: String,
    rows: std::sync::Arc<[RelationalMergeInspectionRow]>,
    artifact_digest: String,
}

impl RelationalMergeInspectionArtifact {
    pub fn from_input(input: RelationalMergeInspectionInput) -> Self {
        let RelationalMergeInspectionInput {
            request,
            lowered_plan,
        } = input;
        let rows = lowered_plan
            .records
            .iter()
            .map(RelationalMergeInspectionRow::from_lowered_record)
            .collect::<Vec<_>>();
        let lowered_plan_digest = merge_inspection_lowered_plan_digest(
            &request,
            &rows,
            lowered_plan.record_count,
            lowered_plan.blocked_count,
            lowered_plan.rejected_count,
        );
        let artifact_digest =
            merge_inspection_artifact_digest(&request, &lowered_plan_digest, &rows);

        Self {
            request,
            lowered_plan_digest,
            rows: std::sync::Arc::from(rows),
            artifact_digest,
        }
    }

    pub fn request(&self) -> &NormalizedRelationalMergeRequest {
        &self.request
    }

    pub fn lowered_plan_digest(&self) -> &str {
        &self.lowered_plan_digest
    }

    pub fn rows(&self) -> &[RelationalMergeInspectionRow] {
        &self.rows
    }

    pub fn artifact_digest(&self) -> &str {
        &self.artifact_digest
    }
}

impl MergePlanningArtifactCore {
    pub fn inspection_input(&self) -> RelationalMergeInspectionInput {
        RelationalMergeInspectionInput::from_planning_artifact(self)
    }
}

impl RelationalMergeInspectionRow {
    fn from_lowered_record(record: &crate::merge::data::policy::LoweredMergePlanRecord) -> Self {
        let decision_kind = lowered_record_decision_kind(&record.record_decision);
        let admission = inspection_admission_for_decision(decision_kind);
        let row_digest = merge_inspection_row_digest(
            &record.record,
            record.target_record.as_ref(),
            &record.classification,
            &record.resolution_class,
            &record.readiness,
            decision_kind,
            record.blocked_reason,
            record.rejected_reason,
            admission,
        );

        Self {
            record: record.record.clone(),
            target_record: record.target_record.clone(),
            classification: record.classification,
            resolution_class: record.resolution_class,
            readiness: record.readiness,
            decision_kind,
            blocked_reason: record.blocked_reason,
            rejected_reason: record.rejected_reason,
            admission,
            row_digest,
        }
    }
}

fn lowered_record_decision_kind(decision: &LoweredRecordDecision) -> LoweredRecordDecisionKind {
    match decision {
        LoweredRecordDecision::Execute(_) => LoweredRecordDecisionKind::Execute,
        LoweredRecordDecision::Block(_) => LoweredRecordDecisionKind::Block,
        LoweredRecordDecision::Reject(_) => LoweredRecordDecisionKind::Reject,
    }
}

fn inspection_admission_for_decision(
    decision_kind: LoweredRecordDecisionKind,
) -> RelationalMergeInspectionAdmission {
    match decision_kind {
        LoweredRecordDecisionKind::Execute => {
            RelationalMergeInspectionAdmission::ExecutionAdmissible
        }
        LoweredRecordDecisionKind::Block | LoweredRecordDecisionKind::Reject => {
            RelationalMergeInspectionAdmission::ExecutionDenied
        }
    }
}
