use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_wal::WalLsnRange;

use crate::physical_runtime::IntegrityAdmittedRecoveryWalFrame;

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
    freshness: &super::super::PhysicalRecoveryFreshnessAuthority,
    checkpoint_basis: Option<&super::StoreRecoveryCheckpointBindingBasis>,
    media: &AdmittedRecoveryFilesystemMedia,
    checkpoint: &VerifiedCheckpointStream,
    wal_frames: impl IntoIterator<Item = &'frame IntegrityAdmittedRecoveryWalFrame>,
    maximum_operation_bindings: u64,
    maximum_redo_bytes: u64,
) -> Result<StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure> {
    super::sample_binding_from_frames(
        freshness,
        checkpoint_basis,
        media,
        checkpoint,
        wal_frames,
        maximum_operation_bindings,
        maximum_redo_bytes,
    )
}
