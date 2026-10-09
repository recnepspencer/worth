use super::*;
use crate::physical_runtime::instance::RebuildReadAdmission;
use worth_store_layout_indexes::AccessLaneClassification;

impl PhysicalSchedulerDemand {
    pub(in crate::physical_runtime) fn rebuild_read(
        ready: ReadyPhysicalWork,
        admitted: RebuildReadAdmission,
    ) -> Result<
        (
            Self,
            IoSchedulerBackendCapabilityAdmission,
            worth_foundational::FoundationalPolicyAdmissionReceipt,
        ),
        PhysicalSchedulerDenial,
    > {
        ready
            .require_consumer_active()
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let contract = admitted.shape.contract();
        let intent = ready.intent();
        let valid_work = match admitted.bytes {
            Some(bytes) => {
                let [coordinate] = intent.scope().coordinates() else {
                    return Err(PhysicalSchedulerDenial::RebuildAdmissionMismatch);
                };
                intent.operation() == PhysicalWorkOperationFamily::ArtifactRangeRead
                    && intent.scope().inspection_target().is_none()
                    && u64::from(coordinate.length()) == bytes.get()
            }
            None => {
                intent.operation() == PhysicalWorkOperationFamily::ArtifactMetadataRead
                    && intent.scope().coordinates().is_empty()
                    && intent.scope().inspection_target().is_none()
            }
        };
        // RebuildReadShape can only be minted by the record-serving owner from
        // the layout RebuildRead declaration; this consumes that carried proof.
        if !valid_work
            || contract.lane() != AccessLaneClassification::Maintenance
            || admitted.lease.class()
                != worth_store_io_scheduler::BackgroundIoPressureClass::RepairScan
        {
            return Err(PhysicalSchedulerDenial::RebuildAdmissionMismatch);
        }
        ready
            .admit_scheduler_pressure(super::super::PhysicalWorkPressureClass::BackgroundRebuild)
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        Ok((
            Self {
                ready,
                work: lower_background_queue_lease(admitted.lease),
                capacity: Some(admitted.capacity),
            },
            admitted.backend,
            admitted.policy,
        ))
    }
}
