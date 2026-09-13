use worth_store_physical_backend::{
    BackendDurabilityProfile, PosixFileFsyncDirFsyncProfile, WalAppendObservationScope,
    WalAppendReceipt, WalDurabilityObservation,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange, WalSegmentGeneration, WalSegmentId};

pub fn completed_posix_receipt() -> WalAppendReceipt<PosixFileFsyncDirFsyncProfile> {
    completed_posix_receipt_for_range(100, 101)
}

pub fn completed_posix_receipt_for_range(
    start: u64,
    end_exclusive: u64,
) -> WalAppendReceipt<PosixFileFsyncDirFsyncProfile> {
    observed_receipt(
        WalAppendObservationScope::new(
            WalSegmentId::new(42).unwrap(),
            WalSegmentGeneration::new(7).unwrap(),
            WalLsnRange::new(
                LogSequenceNumber::new(start),
                LogSequenceNumber::new(end_exclusive),
            )
            .unwrap(),
            format!("page-lsn-frame-digest-{start}-{end_exclusive}"),
            4096,
        )
        .unwrap(),
    )
}

pub fn completed_posix_observation() -> WalDurabilityObservation<PosixFileFsyncDirFsyncProfile> {
    WalDurabilityObservation::from_append_receipt(completed_posix_receipt()).unwrap()
}

fn observed_receipt(
    scope: WalAppendObservationScope,
) -> WalAppendReceipt<PosixFileFsyncDirFsyncProfile> {
    WalAppendReceipt::from_certification_observation(
        scope,
        4096,
        PosixFileFsyncDirFsyncProfile::REQUIRED_BARRIERS,
        None,
    )
}
