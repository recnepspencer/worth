use worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope;

use super::{configuration::FIXTURE_FRAME_BYTES, PhysicalResidencyStoreWorld};

// Materialization, encoded candidate, and recovery projection frames.
const INLINE_RECORD_WORKING_SET_FRAMES: u64 = 3;
const ROUTING_WORKING_SET_FRAMES: u64 = 64 * 3 + 4;
const ONE_INLINE_RECORD_PUBLICATION_BYTES: u64 =
    (INLINE_RECORD_WORKING_SET_FRAMES + ROUTING_WORKING_SET_FRAMES) * FIXTURE_FRAME_BYTES;

#[test]
fn real_store_record_publication_and_borrowed_read_fit_the_fixture_envelope() {
    let world = PhysicalResidencyStoreWorld::initialize("record-publication-journey").unwrap();
    let payload = b"real-store-record-publication";

    let (basis, observed_bytes) = world
        .with_record_chunk(payload, |serving, chunk| {
            assert_eq!(chunk.basis().store_identity(), serving.store_identity());
            assert_eq!(chunk.bytes(), payload);
            (chunk.basis(), chunk.bytes().len())
        })
        .unwrap();

    assert_eq!(basis.store_identity(), world.serving().store_identity());
    assert_eq!(observed_bytes, payload.len());
    let counters = world.serving().residency_observation().counters();
    assert_eq!(counters.active_operation_bytes(), 0);
    assert_eq!(
        counters.active_operation_bytes_for(Scope::ForegroundWrite),
        0
    );
    assert_eq!(
        counters.active_operation_bytes_for(Scope::ForegroundRead),
        0
    );
    assert_eq!(
        counters.peak_operation_bytes_for(Scope::ForegroundWrite),
        ONE_INLINE_RECORD_PUBLICATION_BYTES,
    );
    assert_eq!(
        counters.peak_operation_bytes_for(Scope::ForegroundRead),
        FIXTURE_FRAME_BYTES,
    );
    assert_eq!(
        counters.peak_operation_bytes(),
        ONE_INLINE_RECORD_PUBLICATION_BYTES,
    );
    assert!(!world.close().residency().requires_inspection());
}
