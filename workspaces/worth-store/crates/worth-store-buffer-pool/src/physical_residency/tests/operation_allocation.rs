use super::*;

#[path = "operation_allocation/resize.rs"]
mod resize;

#[test]
fn candidate_dirty_posture_and_operation_allocations_are_exact() {
    let identity = store(5);
    let grant_bytes = candidate_batch_bytes(1);
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, grant_bytes, 4)).unwrap();
    let key = PhysicalFrameKey::new(identity, coordinate(1, 20));
    let grant = pool
        .begin_foreground_write_operation(nonzero_bytes(grant_bytes))
        .unwrap();
    let dirty = pool
        .materialize_dirty_candidate(&grant, key, |bytes| bytes.fill(9))
        .unwrap();
    assert_eq!(pool.counters().dirty_frames(), 1);
    assert_eq!(grant.bytes(), grant_bytes);
    let observation = grant.observation();
    assert_eq!(observation.store(), identity);
    assert_eq!(observation.pool(), pool.incarnation());
    assert_eq!(
        observation.scope(),
        PhysicalOperationAllocationScope::ForegroundWrite
    );
    assert_eq!(observation.bytes(), grant_bytes);
    assert_eq!(
        observation
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::ForegroundWrite),
        grant_bytes
    );
    assert_eq!(
        pool.begin_operation(READ_SCOPE, nonzero_bytes(1))
            .unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::OperationBytes,
                scope: READ_SCOPE,
                requested: 1,
                current: grant_bytes,
                limit: grant_bytes,
            },
        )),
    );
    dirty.discard_candidate().unwrap();
    drop(grant);
    let counters = pool.counters();
    assert_eq!(counters.dirty_frames(), 0);
    assert_eq!(counters.writebacks(), 0);
    assert_eq!(counters.candidate_publications(), 0);
    assert_eq!(counters.active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn same_store_foreign_incarnation_grant_opens_no_admission_surface() {
    let identity = store(6);
    let operation_bytes = candidate_batch_bytes(1) + 20;
    let foreign_pool =
        PhysicalResidencyPool::open(identity, limits(1024, 2, 1, operation_bytes, 4)).unwrap();
    let governed_pool =
        PhysicalResidencyPool::open(identity, limits(1024, 2, 1, operation_bytes, 4)).unwrap();
    let foreign_grant = candidate_allocation(&foreign_pool, 1);
    let foreign_read = foreign_pool
        .begin_foreground_read_operation(nonzero_bytes(20))
        .unwrap();
    let key = PhysicalFrameKey::new(identity, coordinate(1, 20));
    let candidate = PhysicalCandidateFrameKey::fragment(key);
    let before = governed_pool.counters();

    assert_eq!(
        governed_pool.access_frame(&foreign_grant, key).unwrap_err(),
        PhysicalResidencyDenial::AllocationGrantMismatch
    );
    assert_eq!(
        governed_pool
            .materialize_dirty_candidate(&foreign_grant, key, |bytes| bytes.fill(9))
            .unwrap_err(),
        PhysicalResidencyDenial::AllocationGrantMismatch
    );
    assert_eq!(
        governed_pool
            .reserve_candidate_frames(&foreign_grant, &[candidate])
            .unwrap_err(),
        PhysicalResidencyDenial::AllocationGrantMismatch
    );
    assert_eq!(
        governed_pool
            .admit_prefetch(foreign_read, key.coordinate())
            .unwrap_err(),
        PhysicalResidencyDenial::AllocationGrantMismatch
    );

    let after = governed_pool.counters();
    assert_eq!(after.source_loads(), before.source_loads());
    assert_eq!(after.resident_bytes(), before.resident_bytes());
    assert_eq!(after.pinned_frames(), before.pinned_frames());
    assert_eq!(after.dirty_frames(), before.dirty_frames());
    assert_eq!(after.candidate_frames(), before.candidate_frames());
    assert_eq!(
        after.speculative_attempts(PhysicalSpeculativeWorkKind::Prefetch),
        before.speculative_attempts(PhysicalSpeculativeWorkKind::Prefetch)
    );
}

#[test]
fn foreign_incarnation_authority_wins_over_malformed_candidate_inputs() {
    let identity = store(7);
    let operation_bytes = candidate_batch_bytes(1);
    let foreign_pool =
        PhysicalResidencyPool::open(identity, limits(1024, 2, 1, operation_bytes, 4)).unwrap();
    let governed_pool =
        PhysicalResidencyPool::open(identity, limits(1024, 2, 1, operation_bytes, 4)).unwrap();
    let foreign_grant = candidate_allocation(&foreign_pool, 1);
    let governed_grant = candidate_allocation(&governed_pool, 1);
    let declared_key = PhysicalFrameKey::new(identity, coordinate(1, 20));
    let declared_candidate = PhysicalCandidateFrameKey::fragment(declared_key);
    let batch = governed_pool
        .reserve_candidate_frames(&governed_grant, &[declared_candidate])
        .unwrap();
    let before = governed_pool.counters();

    assert_eq!(
        governed_pool
            .materialize_dirty_candidate(&foreign_grant, declared_key, |bytes| bytes.fill(9))
            .unwrap_err(),
        PhysicalResidencyDenial::AllocationGrantMismatch
    );
    assert_eq!(batch.keys.len(), 1);

    assert_eq!(governed_pool.counters(), before);
}

#[test]
fn undersized_grant_denies_before_candidate_metadata_or_publication_activity() {
    let identity = store(8);
    let demand = candidate_batch_bytes(1);
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, demand, 4)).unwrap();
    let tiny = pool
        .begin_foreground_write_operation(NonZeroU64::MIN)
        .unwrap();
    let candidate =
        PhysicalCandidateFrameKey::fragment(PhysicalFrameKey::new(identity, coordinate(1, 20)));
    let before = pool.counters();

    assert_eq!(
        pool.reserve_candidate_frames(&tiny, &[candidate])
            .unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::OperationBytes,
                scope: WRITE_SCOPE,
                requested: demand,
                current: 0,
                limit: 1,
            },
        ))
    );

    let after = pool.counters();
    assert_eq!(after.candidate_frames(), before.candidate_frames());
    assert_eq!(
        after.candidate_publications(),
        before.candidate_publications()
    );
    assert_eq!(
        after.active_loading_frames(),
        before.active_loading_frames()
    );
    assert_eq!(after.denials(), before.denials() + 1);
    drop(tiny);

    let exact = candidate_allocation(&pool, 1);
    let batch = pool.reserve_candidate_frames(&exact, &[candidate]).unwrap();
    drop(batch);
    drop(exact);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
}

#[test]
fn independent_batches_cannot_double_spend_one_grant_and_release_exactly() {
    let identity = store(9);
    let one_batch = candidate_batch_bytes(1);
    let one_two_candidate_batch = candidate_batch_bytes(2);
    assert!(one_two_candidate_batch < one_batch * 2);
    let pool =
        PhysicalResidencyPool::open(identity, limits(1024, 2, 1, one_two_candidate_batch, 4))
            .unwrap();
    let grant = pool
        .begin_foreground_write_operation(NonZeroU64::new(one_two_candidate_batch).unwrap())
        .unwrap();
    let first =
        PhysicalCandidateFrameKey::fragment(PhysicalFrameKey::new(identity, coordinate(1, 20)));
    let second =
        PhysicalCandidateFrameKey::fragment(PhysicalFrameKey::new(identity, coordinate(2, 20)));
    let first_batch = pool.reserve_candidate_frames(&grant, &[first]).unwrap();
    let before_denial = pool.counters();

    assert_eq!(
        pool.reserve_candidate_frames(&grant, &[second])
            .unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::OperationBytes,
                scope: WRITE_SCOPE,
                requested: one_batch,
                current: one_batch,
                limit: one_two_candidate_batch,
            },
        ))
    );
    assert_eq!(
        pool.counters().candidate_frames(),
        before_denial.candidate_frames()
    );

    drop(first_batch);
    let second_batch = pool.reserve_candidate_frames(&grant, &[second]).unwrap();
    drop(second_batch);
    drop(grant);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
}

#[test]
fn candidate_projection_admission_is_linearized_against_pool_close() {
    let identity = store(10);
    let demand = candidate_batch_bytes(1);
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, demand, 4)).unwrap();
    let grant = candidate_allocation(&pool, 1);
    let candidate =
        PhysicalCandidateFrameKey::fragment(PhysicalFrameKey::new(identity, coordinate(1, 20)));
    pool.close();
    let before = pool.counters();

    assert_eq!(
        pool.begin_candidate_batch(&grant, NonZeroUsize::MIN)
            .unwrap_err(),
        PhysicalResidencyDenial::PoolClosed
    );
    let after = pool.counters();
    assert_eq!(after.candidate_frames(), before.candidate_frames());
    assert_eq!(
        after.candidate_publications(),
        before.candidate_publications()
    );
    assert_eq!(
        after.active_loading_frames(),
        before.active_loading_frames()
    );
    assert_eq!(after.denials(), before.denials() + 1);
    drop(grant);
    assert_eq!(pool.counters().active_operation_bytes(), 0);

    let racing_pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, demand, 4)).unwrap();
    let racing_grant = candidate_allocation(&racing_pool, 1);
    let admission = racing_pool
        .begin_candidate_batch(&racing_grant, NonZeroUsize::MIN)
        .unwrap();
    racing_pool.close();
    assert_eq!(
        admission.reserve(&[candidate]).unwrap_err(),
        PhysicalResidencyDenial::PoolClosed
    );
    drop(racing_grant);
    assert_eq!(racing_pool.counters().active_operation_bytes(), 0);
}
