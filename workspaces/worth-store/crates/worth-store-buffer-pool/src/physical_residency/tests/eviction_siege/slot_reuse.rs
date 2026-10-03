use super::*;

fn bounded_key(identity: StableStoreIdentity, block: u64) -> PhysicalBoundedFrameKey {
    PhysicalBoundedFrameKey::new(
        identity,
        coordinate(block, 32).artifact(),
        NonZeroU32::new(32).unwrap(),
    )
}

fn load_bounded(
    pool: &PhysicalResidencyPool,
    read: &OperationAllocationGrant,
    key: PhysicalBoundedFrameKey,
    byte: u8,
) -> PhysicalFrameLease {
    let PhysicalBoundedFrameAccess::Fault(owner) = pool.access_bounded_frame(read, key).unwrap()
    else {
        panic!("the absent bounded artifact must fault");
    };
    owner
        .load(|_| Ok::<_, ()>(32), |bytes| fill(bytes, byte))
        .unwrap()
}

#[test]
fn reused_slots_preserve_bounded_aliases_and_exact_eviction_order() {
    let identity = store(34);
    let pool = PhysicalResidencyPool::open(identity, limits(64, 2, 1, 64, 2)).unwrap();
    let read = allocation(&pool, READ_SCOPE);
    let metadata = pool.counters().metadata_bytes();
    let first_key = PhysicalFrameKey::new(identity, coordinate(1, 32));
    let second_key = PhysicalFrameKey::new(identity, coordinate(2, 32));
    drop(load_bounded(&pool, &read, bounded_key(identity, 1), 1));
    drop(
        expect_fault(&pool, &read, second_key)
            .load(|bytes| fill(bytes, 2))
            .unwrap(),
    );
    let first = expect_hit(&pool, &read, first_key);
    let first_generation = first.resident_generation();
    let third_key = PhysicalFrameKey::new(identity, coordinate(3, 32));
    let third = expect_fault(&pool, &read, third_key)
        .load(|bytes| fill(bytes, 3))
        .unwrap();
    assert_eq!(&*first, &[1; 32]);
    assert_eq!(&*third, &[3; 32]);
    assert_eq!(pool.counters().evictions(), 1);
    assert!(matches!(
        pool.access_frame(&read, second_key),
        Err(PhysicalResidencyDenial::Pressure(_))
    ));
    assert!(matches!(
        pool.access_bounded_frame(&read, bounded_key(identity, 2)),
        Err(PhysicalResidencyDenial::Pressure(_))
    )); // The removed exact artifact cannot alias its slot's new occupant.
    drop(third);
    drop(first);
    // Third is oldest, so its recycled slot is now a bounded resident.
    drop(load_bounded(&pool, &read, bounded_key(identity, 4), 4));
    let first = expect_hit(&pool, &read, first_key);
    assert_eq!(first.resident_generation(), first_generation);
    drop(first); // Fourth is oldest after the exact alias repins First.
    let fifth_key = PhysicalFrameKey::new(identity, coordinate(5, 32));
    let fifth = expect_fault(&pool, &read, fifth_key)
        .load(|bytes| fill(bytes, 5))
        .unwrap();
    let first = expect_hit(&pool, &read, first_key);
    assert_eq!(&*first, &[1; 32]);
    assert_eq!(&*fifth, &[5; 32]);
    assert!(matches!(
        pool.access_bounded_frame(&read, bounded_key(identity, 4)),
        Err(PhysicalResidencyDenial::Pressure(_))
    )); // Neither bounded nor exact aliases survive the bounded slot's reuse.
    assert!(matches!(
        pool.access_frame(&read, PhysicalFrameKey::new(identity, coordinate(4, 32))),
        Err(PhysicalResidencyDenial::Pressure(_))
    ));
    let after = pool.counters();
    assert_eq!(after.evictions(), 3);
    assert_eq!(after.source_loads(), 5);
    assert_eq!(after.frame_entries(), 2);
    assert_eq!(after.metadata_bytes(), metadata);
}
