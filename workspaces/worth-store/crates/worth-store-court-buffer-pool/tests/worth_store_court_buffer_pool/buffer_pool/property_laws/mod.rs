//! Stateful model-checking property suite for `PhysicalResidencyPool`:
//! arbitrary pin/unpin/dirty/replacement/writeback/drain/invalidate sequences
//! predicting exact success vs typed `PhysicalResidencyDenial` from model state
//! and asserting page byte identity, exact drain counts, writeback byte-claim
//! and clean-transition semantics, residency limits, and foreign-authority denials.
#[path = "model.rs"]
mod model;

use self::model::{ModelState, PAGE_COUNT, POOL_CAPACITY};
use proptest::prelude::*;
use std::num::{NonZeroU64, NonZeroUsize};
use worth_store_buffer_pool::{
    ForegroundWriteAllocationGrant, PhysicalFrameAccess, PhysicalOperationAllocationScope,
    PhysicalResidencyDenial, PhysicalResidencyPool,
};

use crate::clean_authority::{frame_key, owned_pool, writeback_claim};

#[derive(Debug, Clone)]
enum PoolAction {
    AccessPin {
        page: u64,
    },
    UnpinLease {
        idx: usize,
    },
    MaterializeDirty {
        page: u64,
        byte: u8,
        publish_clean: bool,
    },
    DropDirtyHandle {
        idx: usize,
    },
    ReplacePinnedWithDirty {
        idx: usize,
        byte: u8,
    },
    WritebackFlush {
        page: u64,
    },
    DrainUnpinnedClean,
    InvalidateClean {
        page: u64,
    },
    ProbeForeignAuthority {
        page: u64,
        byte: u8,
    },
}

fn pool_action_strategy() -> impl Strategy<Value = PoolAction> {
    prop_oneof![
        5 => (1_u64..=PAGE_COUNT).prop_map(|page| PoolAction::AccessPin { page }),
        4 => any::<usize>().prop_map(|idx| PoolAction::UnpinLease { idx }),
        3 => (1_u64..=PAGE_COUNT, any::<u8>(), any::<bool>())
            .prop_map(|(page, byte, publish_clean)| PoolAction::MaterializeDirty {
                page,
                byte,
                publish_clean,
            }),
        3 => any::<usize>().prop_map(|idx| PoolAction::DropDirtyHandle { idx }),
        3 => (any::<usize>(), any::<u8>())
            .prop_map(|(idx, byte)| PoolAction::ReplacePinnedWithDirty { idx, byte }),
        3 => (1_u64..=PAGE_COUNT).prop_map(|page| PoolAction::WritebackFlush { page }),
        2 => Just(PoolAction::DrainUnpinnedClean),
        2 => (1_u64..=PAGE_COUNT).prop_map(|page| PoolAction::InvalidateClean { page }),
        1 => (1_u64..=PAGE_COUNT, any::<u8>())
            .prop_map(|(page, byte)| PoolAction::ProbeForeignAuthority { page, byte }),
    ]
}

fn candidate_write_grant(pool: &PhysicalResidencyPool) -> ForegroundWriteAllocationGrant {
    pool.begin_foreground_write_operation(
        PhysicalResidencyPool::candidate_batch_operation_bytes(NonZeroUsize::MIN).unwrap(),
    )
    .expect("single-step foreground write grant must succeed")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn buffer_pool_preserves_page_identity_dirty_writeback_and_pin_refusal(
        store_byte in 1_u8..=250,
        actions in prop::collection::vec(pool_action_strategy(), 1..=48),
    ) {
        let (pool, candidate_clean, writeback_clean) = owned_pool(store_byte);
        let (_, foreign_candidate_clean, foreign_writeback_clean) = owned_pool(store_byte);
        let mut model = ModelState::new();

        for action in actions {
            match action {
                PoolAction::AccessPin { page } => {
                    let key = frame_key(pool.store_identity(), page);
                    let read_grant = pool
                        .begin_operation(
                            PhysicalOperationAllocationScope::ForegroundRead,
                            NonZeroU64::new(8).unwrap(),
                        )
                        .expect("single-step read grant must succeed");
                    let is_resident = model.resident.contains(&page);
                    let can_admit = model.total_leases() < POOL_CAPACITY
                        && (is_resident || model.fixed_frames() < POOL_CAPACITY);
                    let res = pool.access_frame(&read_grant, key);
                    if !can_admit {
                        prop_assert!(
                            matches!(res, Err(PhysicalResidencyDenial::Pressure(_))),
                            "full pool/leases must deny access_frame with Pressure, got {res:?}"
                        );
                    } else if is_resident {
                        let PhysicalFrameAccess::Hit(lease) =
                            res.expect("resident page must hit")
                        else {
                            prop_assert!(false, "resident page {page} must produce Hit");
                            unreachable!();
                        };
                        prop_assert_eq!(lease.key(), key);
                        prop_assert_eq!(&*lease, &model.expected_bytes(page));
                        model.detach_evictable(page);
                        model.clean_leases.push((page, lease));
                    } else {
                        prop_assert!(!model.dirty.contains_key(&page));
                        let PhysicalFrameAccess::Fault(owner) =
                            res.expect("non-resident page with capacity must fault")
                        else {
                            prop_assert!(false, "absent page {page} must produce Fault");
                            unreachable!();
                        };
                        let expected = model.backing[&page];
                        let lease = owner
                            .load::<(), _>(|buf| {
                                buf.copy_from_slice(&expected);
                                Ok(())
                            })
                            .expect("fault load succeeds");
                        prop_assert_eq!(&*lease, &expected);
                        model.admit_new_resident(page);
                        model.clean_leases.push((page, lease));
                    }
                }
                PoolAction::UnpinLease { idx } => {
                    if !model.clean_leases.is_empty() {
                        let pos = idx % model.clean_leases.len();
                        let (page, lease) = model.clean_leases.swap_remove(pos);
                        drop(lease);
                        model.maybe_mark_evictable(page);
                    }
                }
                PoolAction::MaterializeDirty {
                    page,
                    byte,
                    publish_clean,
                } => {
                    let key = frame_key(pool.store_identity(), page);
                    let write_grant = candidate_write_grant(&pool);
                    let res = pool.materialize_dirty_candidate(&write_grant, key, |b| b.fill(byte));
                    if let Some(expected_denial) = model.candidate_denial(page) {
                        let err = res.expect_err("inadmissible dirty candidate must refuse");
                        prop_assert!(
                            expected_denial.matches(err),
                            "expected {expected_denial:?}, got {err:?}"
                        );
                    } else {
                        let dirty = res.expect("admissible dirty candidate must succeed");
                        prop_assert_eq!(dirty.bytes(), &[byte; 8]);
                        model.admit_new_resident(page);
                        if publish_clean {
                            let lease = dirty
                                .complete_candidate_publication(&candidate_clean)
                                .expect("owning authority publishes candidate clean");
                            model.backing.insert(page, [byte; 8]);
                            model.clean_leases.push((page, lease));
                        } else {
                            model.dirty.insert(page, [byte; 8]);
                            model.dirty_handles.push((page, dirty));
                        }
                    }
                }
                PoolAction::DropDirtyHandle { idx } => {
                    if !model.dirty_handles.is_empty() {
                        let pos = idx % model.dirty_handles.len();
                        let (page, handle) = model.dirty_handles.swap_remove(pos);
                        drop(handle);
                        model.maybe_mark_evictable(page);
                    }
                }
                PoolAction::ReplacePinnedWithDirty { idx, byte } => {
                    if !model.clean_leases.is_empty() {
                        let pos = idx % model.clean_leases.len();
                        let (page, lease) = model.clean_leases.swap_remove(pos);
                        let pins_including_this = 1 + model.pin_count(page);
                        let expected_old = model.expected_bytes(page);
                        let write_grant = pool
                            .begin_foreground_write_operation(NonZeroU64::new(8).unwrap())
                            .expect("single-step replacement grant must succeed");
                        let res = lease.begin_dirty_replacement(&write_grant);
                        if model.dirty.len() >= POOL_CAPACITY {
                            prop_assert!(
                                matches!(res, Err(PhysicalResidencyDenial::Pressure(_))),
                                "full dirty frames must deny replacement with Pressure"
                            );
                            model.maybe_mark_evictable(page);
                        } else if pins_including_this > 1 {
                            prop_assert_eq!(res.unwrap_err(), PhysicalResidencyDenial::FramePinned);
                        } else if model.dirty.contains_key(&page) {
                            prop_assert_eq!(res.unwrap_err(), PhysicalResidencyDenial::FrameDirty);
                        } else {
                            let reservation = res.expect("sole clean pin must begin replacement");
                            let dirty = reservation
                                .replace::<(), _>(|old, new| {
                                    assert_eq!(old, &expected_old);
                                    new.fill(byte);
                                    Ok(())
                                })
                                .expect("dirty replacement fill must succeed");
                            prop_assert_eq!(dirty.bytes(), &[byte; 8]);
                            model.dirty.insert(page, [byte; 8]);
                            model.dirty_handles.push((page, dirty));
                        }
                    }
                }
                PoolAction::WritebackFlush { page } => {
                    let key = frame_key(pool.store_identity(), page);
                    let res = writeback_claim(&pool, key);
                    if let Some(expected) = model.dirty.get(&page).copied() {
                        let claim = res.expect("dirty frame must admit writeback claim");
                        prop_assert_eq!(claim.frame_bytes(0), Some(expected.as_slice()));
                        claim
                            .complete_writeback(&writeback_clean)
                            .expect("owning authority completes writeback");
                        model.backing.insert(page, expected);
                        model.dirty.remove(&page);
                        model.maybe_mark_evictable(page);
                    } else {
                        prop_assert_eq!(
                            res.unwrap_err(),
                            PhysicalResidencyDenial::WriteBackFrameNotDirty
                        );
                    }
                }
                PoolAction::DrainUnpinnedClean => {
                    let expected_drained = model.evictable.len() as u64;
                    let drained = pool.drain_unpinned_clean_frames();
                    prop_assert_eq!(
                        drained,
                        expected_drained,
                        "drain must remove every unpinned clean frame"
                    );
                    for evicted in model.evictable.drain(..) {
                        prop_assert!(model.resident.remove(&evicted));
                    }
                }
                PoolAction::InvalidateClean { page } => {
                    let key = frame_key(pool.store_identity(), page);
                    let res = pool.invalidate_clean(key);
                    if !model.resident.contains(&page) {
                        prop_assert_eq!(res, Ok(()));
                    } else if model.pin_count(page) > 0 {
                        prop_assert_eq!(res, Err(PhysicalResidencyDenial::FramePinned));
                    } else if model.dirty.contains_key(&page) {
                        prop_assert_eq!(res, Err(PhysicalResidencyDenial::FrameDirty));
                    } else {
                        prop_assert_eq!(res, Ok(()));
                        model.detach_evictable(page);
                        prop_assert!(model.resident.remove(&page));
                    }
                }
                PoolAction::ProbeForeignAuthority { page, byte } => {
                    let key = frame_key(pool.store_identity(), page);
                    if model.dirty.contains_key(&page) {
                        let claim = writeback_claim(&pool, key)
                            .expect("dirty frame claims writeback before foreign check");
                        prop_assert_eq!(
                            claim.complete_writeback(&foreign_writeback_clean),
                            Err(PhysicalResidencyDenial::WritebackCleanAuthorityMismatch)
                        );
                    } else if model.candidate_denial(page).is_none() {
                        let write_grant = candidate_write_grant(&pool);
                        let dirty = pool
                            .materialize_dirty_candidate(&write_grant, key, |b| b.fill(byte))
                            .expect("admissible candidate materializes before foreign check");
                        model.admit_new_resident(page);
                        prop_assert_eq!(
                            dirty
                                .complete_candidate_publication(&foreign_candidate_clean)
                                .unwrap_err(),
                            PhysicalResidencyDenial::CandidateCleanAuthorityMismatch
                        );
                        model.dirty.insert(page, [byte; 8]);
                    }
                }
            }
            model.assert_invariants(&pool);
        }

        // Clean drain & writeback of remaining dirty pages, then verify spotless shutdown.
        model.clean_leases.clear();
        model.dirty_handles.clear();
        let remaining_dirty: Vec<u64> = model.dirty.keys().copied().collect();
        for page in remaining_dirty {
            let key = frame_key(pool.store_identity(), page);
            let claim = writeback_claim(&pool, key).expect("remaining dirty page claims writeback");
            prop_assert_eq!(claim.frame_bytes(0), Some(model.dirty[&page].as_slice()));
            claim.complete_writeback(&writeback_clean).unwrap();
            model.dirty.remove(&page);
        }
        let shutdown = pool.close();
        prop_assert!(
            !shutdown.requires_inspection(),
            "cleanly flushed pool must shut down cleanly: {shutdown:?}"
        );
    }
}
