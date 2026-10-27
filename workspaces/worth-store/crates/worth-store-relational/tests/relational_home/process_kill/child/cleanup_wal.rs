//! A bounded real WAL prefix for the recovery cleanup interruption fixture.
use std::num::NonZeroU64;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, PhysicalMutationDeadline,
};
use worth_store_relational::RelationalStoreHome;

pub(super) fn cover_segments_at_next_checkpoint(home: &RelationalStoreHome, limit: NonZeroU64) {
    let records = home.certification_serving().anchored_records().unwrap();
    let anchor = AnchoredRecordName::new("relational.oracle.cleanup-wal").unwrap();
    // Sixteen ordinary 64 KiB values at a 512 KiB segment limit: only 1 MiB
    // of owner payload, but more than two segments of real redo. Each value
    // differs, so the publisher cannot answer an identical-publication retry.
    let payload_bytes = usize::try_from(limit.get() / 8).unwrap();
    for generation in 0_u8..16 {
        records
            .publish(AnchoredRecordPublication::new(
                anchor,
                u64::from(generation),
                vec![generation; payload_bytes],
                super::super::super::open_refusals::home_placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ))
            .unwrap();
    }
    eprintln!(
        "cleanup fixture: WAL limit={} bytes; owner payload={} bytes",
        limit,
        payload_bytes * 16
    );
}
