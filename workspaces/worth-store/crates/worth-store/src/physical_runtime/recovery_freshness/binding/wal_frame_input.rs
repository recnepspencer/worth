use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_wal::WalLsnRange;

use crate::physical_runtime::{IntegrityAdmittedRecoveryWalFrame, PhysicalRecoveryCoordination};

use super::{StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure};

pub(super) trait RecoveryWalFrameInput {
    fn recovery_lsn_range(&self) -> WalLsnRange;
    fn recovery_payload(&self) -> &[u8];
}

impl RecoveryWalFrameInput for IntegrityAdmittedRecoveryWalFrame {
    fn recovery_lsn_range(&self) -> WalLsnRange {
        self.lsn_range()
    }

    fn recovery_payload(&self) -> &[u8] {
        self.payload()
    }
}

pub(in crate::physical_runtime::recovery_freshness) fn sample_binding<'frame>(
    covered: super::CheckpointCoveredMembers,
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    basis: super::StoreRecoverySamplingBasis<'_>,
    wal_frames: impl Iterator<Item = &'frame IntegrityAdmittedRecoveryWalFrame> + Clone,
    maximum_operation_bindings: u64,
    maximum_redo_bytes: u64,
    maximum_manifest_cleanup_sampling_bytes: u64,
) -> Result<StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure> {
    super::sampling::sample_binding_from_frames(
        covered,
        coordination,
        media,
        basis,
        wal_frames,
        maximum_operation_bindings,
        maximum_redo_bytes,
        maximum_manifest_cleanup_sampling_bytes,
    )
}
