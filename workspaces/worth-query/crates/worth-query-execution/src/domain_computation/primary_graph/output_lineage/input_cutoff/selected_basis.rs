//! One admitted Native position paired with its exact issued snapshot handle.

use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::verification::InputCutoffVerificationStop;

/// This is preparation evidence for cutoff verification, not mutation authority.
/// Its private fields cannot pair an unrelated position with the issued handle.
pub(in crate::domain_computation::primary_graph) struct PreparedInputCutoffBasis<'snapshot> {
    snapshot: &'snapshot SnapshotHandle,
    positioned: PositionedRelationalSnapshot,
}

impl<'snapshot> PreparedInputCutoffBasis<'snapshot> {
    pub(super) fn into_positioned(self) -> PositionedRelationalSnapshot {
        self.positioned
    }
    pub(in crate::domain_computation::primary_graph) fn prepare(
        runtime: &RelationalRuntime,
        snapshot: &'snapshot SnapshotHandle,
    ) -> Result<Self, InputCutoffVerificationStop> {
        let positioned = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| InputCutoffVerificationStop::SelectedSourceUnavailable)?;
        Ok(Self {
            snapshot,
            positioned,
        })
    }

    pub(super) fn in_runtime(
        &self,
        runtime: &RelationalRuntime,
    ) -> Result<(&SnapshotHandle, &PositionedRelationalSnapshot), InputCutoffVerificationStop> {
        if runtime.runtime_instance_id() != self.positioned.runtime_instance_id() {
            return Err(InputCutoffVerificationStop::SelectedSourceMismatch);
        }
        Ok((self.snapshot, &self.positioned))
    }
}
