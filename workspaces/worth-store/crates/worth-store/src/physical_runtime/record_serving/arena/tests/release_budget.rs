use super::*;

#[test]
fn full_published_index_reconstructs_under_the_same_budget() {
    let mut live =
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 5, 2).unwrap();
    let mut reopened =
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 5, 2).unwrap();
    for offset in [0, 8192, 16384, 24576, 32768] {
        let free = range(1, offset, 4096);
        live.admit_durable_release(free).unwrap();
        reopened.restore_free_range(free).unwrap();
    }
    for owner in [&mut live, &mut reopened] {
        owner.admit_durable_release(range(1, 4096, 4096)).unwrap();
        owner.admit_durable_release(range(1, 12288, 4096)).unwrap();
        assert_eq!(owner.reserve(20480).unwrap().1, range(1, 0, 20480));
    }
}

#[test]
fn recovered_claim_splits_exact_free_range() {
    let mut owner =
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 8, 2).unwrap();
    owner.restore_free_range(range(1, 0, 20480)).unwrap();
    let claim = owner.restore_claim(range(1, 4096, 8192)).unwrap();
    assert_eq!(owner.reserve(4096).unwrap().1, range(1, 0, 4096));
    assert_eq!(owner.reserve(8192).unwrap().1, range(1, 12288, 8192));
    assert_eq!(
        owner.restore_claim(range(1, 4096, 8192)),
        Err(ArenaAllocationDenial::Overlap)
    );
    owner.cancel(claim).unwrap();
    assert_eq!(owner.reserve(8192).unwrap().1, range(1, 4096, 8192));
}

#[test]
fn recovered_unpublished_arena_claim_checks_budget_before_mutation() {
    let mut owner =
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 6, 1).unwrap();
    assert_eq!(
        owner.restore_claim(range(9, 4096, 4096)),
        Err(ArenaAllocationDenial::RangeBudget {
            required: 11,
            maximum: 6,
        })
    );
    let claim = owner.restore_claim(range(2, 4096, 4096)).unwrap();
    assert_eq!(owner.reserve(4096).unwrap().1, range(2, 0, 4096));
    owner.cancel(claim).unwrap();
}

#[test]
fn full_index_admits_coalescing_release_but_not_a_new_isolated_range() {
    let mut allocator =
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 5, 2).unwrap();
    for offset in [0, 8192, 16384, 24576] {
        allocator
            .restore_free_range(range(1, offset, 4096))
            .unwrap();
    }
    let bridge = range(1, 4096, 4096);
    allocator.preflight_release(bridge).unwrap();
    allocator.admit_durable_release(bridge).unwrap();
    assert_eq!(allocator.reserve(12288).unwrap().1, range(1, 0, 12288));
    allocator
        .admit_durable_release(range(1, 32768, 4096))
        .unwrap();
    allocator
        .admit_durable_release(range(1, 40960, 4096))
        .unwrap();
    let isolated = range(1, 49152, 4096);
    assert_eq!(
        allocator.preflight_release(isolated),
        Err(ArenaAllocationDenial::RangeBudget {
            required: 6,
            maximum: 5,
        })
    );
    assert_eq!(
        allocator.admit_durable_release(isolated),
        Err(ArenaAllocationDenial::RangeBudget {
            required: 6,
            maximum: 5,
        })
    );
    // Pressure did not mutate either index; an adjacent bridge still succeeds.
    allocator
        .admit_durable_release(range(1, 20480, 4096))
        .unwrap();
}
