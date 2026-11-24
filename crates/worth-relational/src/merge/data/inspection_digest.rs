use sha2::{Digest, Sha256};

use crate::history::data::coordinates::BranchId;
use crate::merge::data::artifacts::inspection_artifact::{
    RelationalMergeInspectionAdmission, RelationalMergeInspectionRow,
};
use crate::merge::data::conflicts::{DeletionMergeClass, MergeConflictClass};
use crate::merge::data::policy::{
    DeletionExecutionClass, LoweredMergeBlockedReason, LoweredMergeRejectedReason,
    LoweredRecordDecisionKind, MergeExecutionReadiness, MergeResolutionClass,
    TopologyExecutionClass,
};
use crate::merge::data::requests::normalized::{
    NormalizedRelationalMergeRequest, RelationalMergeCorrespondencePosture,
    RelationalMergeRequestFamily, RelationalMergeSchemaReconciliationPosture,
    RelationalMergeTopologyIntent,
};
use crate::merge::data::requests::raw::MergeIntent;
use crate::transactions::data::primitives::RecordRef;

pub(crate) fn merge_inspection_row_digest(
    record: &RecordRef,
    target_record: Option<&RecordRef>,
    classification: &MergeConflictClass,
    resolution_class: &MergeResolutionClass,
    readiness: &MergeExecutionReadiness,
    decision_kind: LoweredRecordDecisionKind,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    rejected_reason: Option<LoweredMergeRejectedReason>,
    admission: RelationalMergeInspectionAdmission,
) -> String {
    let mut bytes = DigestBytes::new("worth.relational.merge.inspection.row.v1");
    bytes.record_ref(record);
    bytes.option_record_ref(target_record);
    bytes.merge_conflict_class(classification);
    bytes.merge_resolution_class(resolution_class);
    bytes.merge_execution_readiness(*readiness);
    bytes.lowered_record_decision_kind(decision_kind);
    bytes.option_blocked_reason(blocked_reason);
    bytes.option_rejected_reason(rejected_reason);
    bytes.inspection_admission(admission);
    bytes.finish()
}

pub(crate) fn merge_inspection_lowered_plan_digest(
    request: &NormalizedRelationalMergeRequest,
    rows: &[RelationalMergeInspectionRow],
    record_count: usize,
    blocked_count: usize,
    rejected_count: usize,
) -> String {
    let mut bytes = DigestBytes::new("worth.relational.merge.inspection.lowered_plan.v1");
    bytes.normalized_merge_request(request);
    bytes.str_list(rows.iter().map(RelationalMergeInspectionRow::row_digest));
    bytes.usize(record_count);
    bytes.usize(blocked_count);
    bytes.usize(rejected_count);
    bytes.finish()
}

pub(crate) fn merge_inspection_artifact_digest(
    request: &NormalizedRelationalMergeRequest,
    lowered_plan_digest: &str,
    rows: &[RelationalMergeInspectionRow],
) -> String {
    let mut bytes = DigestBytes::new("worth.relational.merge.inspection.artifact.v1");
    bytes.normalized_merge_request(request);
    bytes.str(lowered_plan_digest);
    bytes.str_list(rows.iter().map(RelationalMergeInspectionRow::row_digest));
    bytes.finish()
}

struct DigestBytes {
    bytes: Vec<u8>,
}

impl DigestBytes {
    fn new(domain: &'static str) -> Self {
        let mut bytes = Self { bytes: Vec::new() };
        bytes.str(domain);
        bytes
    }

    fn finish(self) -> String {
        let digest = Sha256::digest(self.bytes);
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn tag(&mut self, tag: u8) {
        self.bytes.push(tag);
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn usize(&mut self, value: usize) {
        self.u64(value as u64);
    }

    fn str(&mut self, value: &str) {
        self.usize(value.len());
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn str_list<'a>(&mut self, values: impl Iterator<Item = &'a str>) {
        let values = values.collect::<Vec<_>>();
        self.usize(values.len());
        for value in values {
            self.str(value);
        }
    }

    fn branch_id(&mut self, value: &BranchId) {
        self.str(&value.0);
    }

    fn record_ref(&mut self, value: &RecordRef) {
        match value {
            RecordRef::Entity(id) => {
                self.tag(1);
                self.u32(id.partition_value());
                self.u64(id.local_slot_value());
                self.u32(id.generation_value());
            }
            RecordRef::Relation(id) => {
                self.tag(2);
                self.u32(id.partition_value());
                self.u64(id.local_slot_value());
                self.u32(id.generation_value());
            }
        }
    }

    fn option_record_ref(&mut self, value: Option<&RecordRef>) {
        match value {
            Some(value) => {
                self.tag(1);
                self.record_ref(value);
            }
            None => self.tag(0),
        }
    }

    fn normalized_merge_request(&mut self, value: &NormalizedRelationalMergeRequest) {
        self.branch_id(value.target_branch());
        self.branch_id(value.source_branch());
        self.merge_intent(&value.merge_intent());
        self.request_family(value.family());
        self.correspondence_posture(value.correspondence_posture());
        self.schema_posture(value.schema_reconciliation_posture());
        self.topology_intent(value.topology_intent());
    }

    fn merge_intent(&mut self, value: &MergeIntent) {
        match value {
            MergeIntent::ReconcileIntoTarget => self.tag(1),
        }
    }

    fn request_family(&mut self, value: RelationalMergeRequestFamily) {
        match value {
            RelationalMergeRequestFamily::FullBranchReconciliation => self.tag(1),
        }
    }

    fn correspondence_posture(&mut self, value: RelationalMergeCorrespondencePosture) {
        match value {
            RelationalMergeCorrespondencePosture::Advisory => self.tag(1),
            RelationalMergeCorrespondencePosture::Strict => self.tag(2),
        }
    }

    fn schema_posture(&mut self, value: RelationalMergeSchemaReconciliationPosture) {
        match value {
            RelationalMergeSchemaReconciliationPosture::Participate => self.tag(1),
            RelationalMergeSchemaReconciliationPosture::RequireCompatibility => self.tag(2),
        }
    }

    fn topology_intent(&mut self, value: RelationalMergeTopologyIntent) {
        match value {
            RelationalMergeTopologyIntent::PreserveTopologySemantics => self.tag(1),
            RelationalMergeTopologyIntent::RequireStrictTopologyStability => self.tag(2),
        }
    }

    fn merge_conflict_class(&mut self, value: &MergeConflictClass) {
        match value {
            MergeConflictClass::ExactSharedTruth => self.tag(1),
            MergeConflictClass::SourceOnlyAddition => self.tag(2),
            MergeConflictClass::SchemaDeclaredCorrespondence => self.tag(3),
            MergeConflictClass::Deletion(class) => {
                self.tag(4);
                self.deletion_merge_class(*class);
            }
            MergeConflictClass::DivergentVisibleState => self.tag(5),
            MergeConflictClass::StrategyIntentConflict => self.tag(6),
            MergeConflictClass::RelationEndpointDivergence => self.tag(7),
        }
    }

    fn deletion_merge_class(&mut self, value: DeletionMergeClass) {
        match value {
            DeletionMergeClass::SourceDeletedTargetLive => self.tag(1),
            DeletionMergeClass::SourceLiveTargetDeleted => self.tag(2),
            DeletionMergeClass::DeletedOnBothSides => self.tag(3),
            DeletionMergeClass::DeletedVsModified => self.tag(4),
            DeletionMergeClass::DeletedVsRewired => self.tag(5),
        }
    }

    fn merge_resolution_class(&mut self, value: &MergeResolutionClass) {
        match value {
            MergeResolutionClass::SourceOnlyAddition => self.tag(1),
            MergeResolutionClass::ExactSharedTruth => self.tag(2),
            MergeResolutionClass::SchemaDeclaredCorrespondence => self.tag(3),
            MergeResolutionClass::Deletion(class) => {
                self.tag(4);
                self.deletion_execution_class(*class);
            }
            MergeResolutionClass::Topology(class) => {
                self.tag(5);
                self.topology_execution_class(*class);
            }
            MergeResolutionClass::DivergentVisibleState => self.tag(6),
        }
    }

    fn deletion_execution_class(&mut self, value: DeletionExecutionClass) {
        match value {
            DeletionExecutionClass::SourceDeletedTargetLive => self.tag(1),
            DeletionExecutionClass::SourceLiveTargetDeleted => self.tag(2),
            DeletionExecutionClass::DeletedOnBothSides => self.tag(3),
            DeletionExecutionClass::DeletedVsModified => self.tag(4),
            DeletionExecutionClass::DeletedVsRewired => self.tag(5),
        }
    }

    fn topology_execution_class(&mut self, value: TopologyExecutionClass) {
        match value {
            TopologyExecutionClass::RelationEndpointStable => self.tag(1),
            TopologyExecutionClass::RelationEndpointRewiredLocal => self.tag(2),
            TopologyExecutionClass::RelationEndpointRewiredEscalated => self.tag(3),
            TopologyExecutionClass::TopologyRegionConflict => self.tag(4),
        }
    }

    fn merge_execution_readiness(&mut self, value: MergeExecutionReadiness) {
        match value {
            MergeExecutionReadiness::Admitted => self.tag(1),
            MergeExecutionReadiness::Blocked => self.tag(2),
            MergeExecutionReadiness::Rejected => self.tag(3),
        }
    }

    fn lowered_record_decision_kind(&mut self, value: LoweredRecordDecisionKind) {
        match value {
            LoweredRecordDecisionKind::Execute => self.tag(1),
            LoweredRecordDecisionKind::Block => self.tag(2),
            LoweredRecordDecisionKind::Reject => self.tag(3),
        }
    }

    fn option_blocked_reason(&mut self, value: Option<LoweredMergeBlockedReason>) {
        match value {
            Some(value) => {
                self.tag(1);
                self.lowered_merge_blocked_reason(value);
            }
            None => self.tag(0),
        }
    }

    fn lowered_merge_blocked_reason(&mut self, value: LoweredMergeBlockedReason) {
        match value {
            LoweredMergeBlockedReason::ManualConflictResolutionRequired => self.tag(1),
            LoweredMergeBlockedReason::StrategyIntentConflictRequiresManualResolution => {
                self.tag(2)
            }
            LoweredMergeBlockedReason::MissingVisibleState => self.tag(3),
            LoweredMergeBlockedReason::MissingAncestorValueBasis => self.tag(4),
            LoweredMergeBlockedReason::UnvalidatedSchemaCorrespondence => self.tag(5),
            LoweredMergeBlockedReason::RelationEndpointRewiredLocal => self.tag(6),
            LoweredMergeBlockedReason::RelationEndpointRewiredEscalated => self.tag(7),
            LoweredMergeBlockedReason::TopologyRegionConflict => self.tag(8),
            LoweredMergeBlockedReason::SourceDeletedTargetLive => self.tag(9),
            LoweredMergeBlockedReason::SourceLiveTargetDeleted => self.tag(10),
            LoweredMergeBlockedReason::DeletedOnBothSides => self.tag(11),
            LoweredMergeBlockedReason::DeletedVsModified => self.tag(12),
            LoweredMergeBlockedReason::DeletedVsRewired => self.tag(13),
        }
    }

    fn option_rejected_reason(&mut self, value: Option<LoweredMergeRejectedReason>) {
        match value {
            Some(value) => {
                self.tag(1);
                self.lowered_merge_rejected_reason(value);
            }
            None => self.tag(0),
        }
    }

    fn lowered_merge_rejected_reason(&mut self, value: LoweredMergeRejectedReason) {
        match value {
            LoweredMergeRejectedReason::FailOnConflictPolicy => self.tag(1),
            LoweredMergeRejectedReason::CustomPolicyRejected => self.tag(2),
            LoweredMergeRejectedReason::MixedPolicyRejectClasses => self.tag(3),
        }
    }

    fn inspection_admission(&mut self, value: RelationalMergeInspectionAdmission) {
        match value {
            RelationalMergeInspectionAdmission::ExecutionAdmissible => self.tag(1),
            RelationalMergeInspectionAdmission::ExecutionDenied => self.tag(2),
        }
    }
}
