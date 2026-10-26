use crate::inspection::data::{
    InspectionAccessPath, InspectionAvailability, InspectionDegradation, InspectionOrigin,
    PinStateObservation, ReclaimEligibility, RecordRetentionInspection,
    RetentionExecutionInspection, RetentionInspectionRequest, RetentionInspectionSummary,
    RetentionStateObservation, SnapshotPinInspection,
};
use crate::storage::data::{RecordLifecycleState, RetentionPassOutcome};
use crate::transactions::data::RecordRef;

use super::access::{empty_retention_plan, InspectionAccess};

impl<'runtime> InspectionAccess<'runtime> {
    pub fn retention_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &RetentionInspectionRequest,
    ) -> Result<RetentionInspectionSummary, crate::execution::RelationalExecutionDenialCause> {
        let resident = self.demand_mirror(execution)?;
        let (plan, availability, degradations) = self.inspect_retention_plan(&resident, request);
        Ok(RetentionInspectionSummary {
            current_version_id: self.runtime.current_version_id(),
            active_snapshot_count: plan.active_snapshot_count as u64,
            branch_pinned_entities: plan.branch_pinned_entities as u64,
            replay_pinned_entities: plan.replay_pinned_entities as u64,
            snapshot_pinned_entities: plan.snapshot_pinned_entities as u64,
            branch_pinned_relations: plan.branch_pinned_relations as u64,
            replay_pinned_relations: plan.replay_pinned_relations as u64,
            snapshot_pinned_relations: plan.snapshot_pinned_relations as u64,
            reclaimable_entities: plan.reclaimable_entities as u64,
            reclaimable_relations: plan.reclaimable_relations as u64,
            origin: InspectionOrigin::RetentionState,
            access_path: InspectionAccessPath::DirectLookup,
            availability,
            degradations,
        })
    }

    pub fn inspect_record_retention(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: RecordRef,
    ) -> Result<Option<RecordRetentionInspection>, crate::execution::RelationalExecutionDenialCause>
    {
        let resident = self.demand_mirror(execution)?;
        let version_id = self.runtime.current_version_id();
        let surface = match &target {
            RecordRef::Entity(entity_id) => self.current_entity_slot_surface(&resident, *entity_id),
            RecordRef::Relation(relation_id) => {
                self.current_relation_slot_surface(&resident, *relation_id)
            }
        };
        Ok(surface.map(|surface| {
            self.record_retention_inspection(
                target,
                surface.lifecycle,
                surface.snapshot_pins,
                surface.branch_pins,
                surface.replay_pins,
                version_id,
            )
        }))
    }

    pub fn inspect_snapshot_pinning(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Result<Option<SnapshotPinInspection>, crate::execution::RelationalExecutionDenialCause>
    {
        Ok(self
            .inspect_snapshot(execution, handle)?
            .map(|snapshot| SnapshotPinInspection {
                snapshot,
                origin: InspectionOrigin::VisibilitySnapshot,
                access_path: InspectionAccessPath::SnapshotRead,
                availability: InspectionAvailability::Direct,
            }))
    }

    pub fn inspect_retention_execution(
        &self,
        outcome: RetentionPassOutcome,
    ) -> RetentionExecutionInspection {
        RetentionExecutionInspection {
            outcome,
            origin: InspectionOrigin::RetentionState,
            access_path: InspectionAccessPath::DirectLookup,
        }
    }

    fn record_retention_inspection(
        &self,
        target: RecordRef,
        lifecycle: RecordLifecycleState,
        snapshot_pins: u32,
        branch_pins: u32,
        replay_pins: u32,
        version_id: crate::identity::data::VersionId,
    ) -> RecordRetentionInspection {
        let reclaim_eligibility = if !self.auto_reclaim_deleted_records() {
            ReclaimEligibility::BlockedByPolicy
        } else if snapshot_pins > 0 {
            ReclaimEligibility::BlockedBySnapshotPins
        } else if branch_pins > 0 {
            ReclaimEligibility::BlockedByBranchPins
        } else if replay_pins > 0 {
            ReclaimEligibility::BlockedByReplayPins
        } else if lifecycle == RecordLifecycleState::Reclaimable {
            ReclaimEligibility::EligibleNow
        } else {
            ReclaimEligibility::BlockedByRetentionFence
        };
        RecordRetentionInspection {
            state: RetentionStateObservation {
                target: target.clone(),
                lifecycle,
            },
            pins: PinStateObservation {
                target,
                snapshot_pins,
                branch_pins,
                replay_pins,
            },
            reclaim_eligibility,
            historical_availability: crate::inspection::data::HistoricalAvailabilityObservation {
                version_id,
                availability: if lifecycle == RecordLifecycleState::Reusable {
                    InspectionAvailability::UnavailableByRetention
                } else {
                    InspectionAvailability::Direct
                },
                retained_directly: lifecycle != RecordLifecycleState::Reusable,
            },
        }
    }

    fn inspect_retention_plan(
        &self,
        edition: &crate::runtime::ResidentPartitionEdition,
        request: &RetentionInspectionRequest,
    ) -> (
        crate::storage::data::RetentionPlan,
        InspectionAvailability,
        Vec<InspectionDegradation>,
    ) {
        let retention_fence = self.retention_fence_version();
        let mut branch_pinned_entities = 0;
        let mut replay_pinned_entities = 0;
        let mut snapshot_pinned_entities = 0;
        let mut branch_pinned_relations = 0;
        let mut replay_pinned_relations = 0;
        let mut snapshot_pinned_relations = 0;
        let mut reclaimable_entities = 0;
        let mut reclaimable_relations = 0;
        let mut entity_slot_scans = 0_u64;
        let mut relation_slot_scans = 0_u64;
        let mut work_units = 0_u64;
        // One edition for the whole retention sweep. This inspection only
        // reads, so pinning is safe here and keeps a scan of S slots at one
        // substrate acquisition instead of one per slot surface.
        for (partition_id, _) in edition.entries() {
            for slot in crate::storage::access::record_slots_in::<
                crate::storage::substrate::EntityRecordKind,
            >(edition, partition_id)
            {
                entity_slot_scans += 1;
                work_units += 1;
                if work_units > request.max_work_units {
                    self.count_retention_work(entity_slot_scans, relation_slot_scans);
                    self.count_budget_refusal();
                    return (
                        empty_retention_plan(retention_fence),
                        InspectionAvailability::UnavailableByBudget,
                        vec![InspectionDegradation::WorkBudgetExceeded],
                    );
                }
                if entity_slot_scans > request.max_entity_slots_scanned {
                    self.count_retention_work(entity_slot_scans, relation_slot_scans);
                    self.count_budget_refusal();
                    return (
                        empty_retention_plan(retention_fence),
                        InspectionAvailability::UnavailableByBudget,
                        vec![InspectionDegradation::EntitySlotBudgetExceeded],
                    );
                }
                if let Some(surface) = crate::storage::access::record_slot_surface_in::<
                    crate::storage::substrate::EntityRecordKind,
                >(edition, partition_id, slot)
                {
                    if surface.branch_pins > 0 {
                        branch_pinned_entities += 1;
                    }
                    if surface.replay_pins > 0 {
                        replay_pinned_entities += 1;
                    }
                    if surface.snapshot_pins > 0 {
                        snapshot_pinned_entities += 1;
                    }
                    if surface.lifecycle == RecordLifecycleState::Reclaimable {
                        reclaimable_entities += 1;
                    }
                }
            }
            for slot in crate::storage::access::record_slots_in::<
                crate::storage::substrate::RelationRecordKind,
            >(edition, partition_id)
            {
                relation_slot_scans += 1;
                work_units += 1;
                if work_units > request.max_work_units {
                    self.count_retention_work(entity_slot_scans, relation_slot_scans);
                    self.count_budget_refusal();
                    return (
                        empty_retention_plan(retention_fence),
                        InspectionAvailability::UnavailableByBudget,
                        vec![InspectionDegradation::WorkBudgetExceeded],
                    );
                }
                if relation_slot_scans > request.max_relation_slots_scanned {
                    self.count_retention_work(entity_slot_scans, relation_slot_scans);
                    self.count_budget_refusal();
                    return (
                        empty_retention_plan(retention_fence),
                        InspectionAvailability::UnavailableByBudget,
                        vec![InspectionDegradation::RelationSlotBudgetExceeded],
                    );
                }
                if let Some(surface) = crate::storage::access::record_slot_surface_in::<
                    crate::storage::substrate::RelationRecordKind,
                >(edition, partition_id, slot)
                {
                    if surface.branch_pins > 0 {
                        branch_pinned_relations += 1;
                    }
                    if surface.replay_pins > 0 {
                        replay_pinned_relations += 1;
                    }
                    if surface.snapshot_pins > 0 {
                        snapshot_pinned_relations += 1;
                    }
                    if surface.lifecycle == RecordLifecycleState::Reclaimable {
                        reclaimable_relations += 1;
                    }
                }
            }
        }
        self.count_retention_work(entity_slot_scans, relation_slot_scans);
        (
            crate::storage::data::RetentionPlan {
                retention_fence_version: retention_fence,
                active_snapshot_count: self.active_snapshot_count(),
                branch_pinned_entities,
                replay_pinned_entities,
                snapshot_pinned_entities,
                branch_pinned_relations,
                replay_pinned_relations,
                snapshot_pinned_relations,
                reclaimable_entities,
                reclaimable_relations,
            },
            InspectionAvailability::Direct,
            Vec::new(),
        )
    }
}
