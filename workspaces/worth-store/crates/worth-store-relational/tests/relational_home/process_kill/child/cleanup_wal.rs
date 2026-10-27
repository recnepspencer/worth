//! A bounded real WAL prefix for the recovery cleanup interruption fixture.
use std::num::NonZeroU64;
use worth_store::physical_runtime::{
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial, RecordAppendBatch,
};
use worth_store_relational::RelationalStoreHome;

pub(super) fn cover_segments_at_next_checkpoint(home: &RelationalStoreHome, limit: NonZeroU64) {
    // All four anchors belong to the home. Append unanchored records through
    // Store's existing certification publication driver instead. Sixteen
    // 64 KiB records at a 512 KiB limit give 1 MiB of owner payload and more
    // than two segments of real redo without changing any Relational anchor.
    let payload_bytes = usize::try_from(limit.get() / 8).unwrap();
    for generation in 0_u8..16 {
        home.certification_serving()
            .certification_publish_single_durable_mutation(
                super::super::super::open_refusals::home_placement(),
                PhysicalManifestCapacityTransition::PreserveCurrent,
                PhysicalMutationIdempotencyMaterial::new([generation; 32]),
                RecordAppendBatch::try_from_iter([vec![generation; payload_bytes]]).unwrap(),
            );
    }
    eprintln!(
        "cleanup fixture: WAL limit={} bytes; owner payload={} bytes",
        limit,
        payload_bytes * 16
    );
}
