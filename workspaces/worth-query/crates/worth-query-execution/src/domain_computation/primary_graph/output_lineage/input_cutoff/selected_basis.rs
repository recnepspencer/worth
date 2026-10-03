//! One admitted Native position paired with its exact issued snapshot handle.

use worth_relational::facade::{
    mvcc::CompanionPreflightStop,
    runtime::{
        PositionedRelationalSnapshot, RelationalRuntime, RelationalSnapshotPositionAdmissionStop,
    },
    snapshots::SnapshotHandle,
};

use super::super::invalidation::InvalidationEditAdmission;
use super::verification::InputCutoffVerificationStop;

/// This is preparation evidence for cutoff verification, not mutation authority.
/// Its private fields cannot pair an unrelated position with the issued handle.
pub(in crate::domain_computation::primary_graph) struct PreparedInputCutoffBasis<'snapshot> {
    snapshot: &'snapshot SnapshotHandle,
    positioned: PositionedRelationalSnapshot,
}

impl<'snapshot> PreparedInputCutoffBasis<'snapshot> {
    pub(in crate::domain_computation::primary_graph) fn prepare(
        runtime: &RelationalRuntime,
        snapshot: &'snapshot SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, InputCutoffVerificationStop> {
        let carrier = std::mem::size_of::<Self>() as u64;
        admission.charge_external_work(2 * carrier + 4)?;
        admission.admit_read_scratch(carrier)?;
        let positioned = runtime
            .read_truth()
            .positioned_snapshot_admitted(snapshot, |work, bytes| {
                admission.charge_external_work(work)?;
                admission.admit_read_scratch(bytes)
            })
            .map_err(|stop| match stop {
                RelationalSnapshotPositionAdmissionStop::Admission(stop) => {
                    InputCutoffVerificationStop::Admission(stop)
                }
                RelationalSnapshotPositionAdmissionStop::AccountingOverflow => {
                    InputCutoffVerificationStop::Admission(
                        CompanionPreflightStop::WorkCounterOverflow,
                    )
                }
                RelationalSnapshotPositionAdmissionStop::Position(stop) => {
                    InputCutoffVerificationStop::SelectedSourceUnavailable(stop)
                }
            })?;
        Ok(Self {
            snapshot,
            positioned,
        })
    }

    pub(super) fn in_runtime(
        &self,
        runtime: &RelationalRuntime,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(&SnapshotHandle, &PositionedRelationalSnapshot), InputCutoffVerificationStop> {
        admission.charge_external_work(2 * std::mem::size_of::<u64>() as u64 + 4)?;
        if runtime.runtime_instance_id() != self.positioned.runtime_instance_id() {
            return Err(InputCutoffVerificationStop::SelectedSourceMismatch);
        }
        Ok((self.snapshot, &self.positioned))
    }
}
