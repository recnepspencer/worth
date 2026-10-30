pub(crate) use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
pub(crate) use {
    worth_store::physical_runtime::PhysicalRecordChunkView,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) fn with_record_chunk<R>(
    label: &str,
    payload: &[u8],
    run: impl FnOnce(&ServingPhysicalRuntime, PhysicalRecordChunkView<'_>) -> R,
) -> R {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize(label)
        .expect("stable-read certification requires a real admitted Store");
    let result = world
        .with_record_chunk(payload, run)
        .expect("stable-read certification requires a published Store record chunk");
    assert!(
        !world.close().residency().requires_inspection(),
        "the real Store fixture must close without residency inspection"
    );
    result
}
