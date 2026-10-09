//! Physical typed payload custody; nested payload heaps remain uncharged here.
use std::{
    alloc::Layout,
    cell::Cell,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialKind as Kind,
    ExecutionAllocationPolicy as Policy, ExecutionArrayBuilder, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionResourceLease, LeaseDenial, LeaseRequest, MemoryLimitDenial,
    MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static SERIAL: Mutex<()> = Mutex::new(());
fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: None,
        })
        .unwrap()
    })
}
fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn typed_layout_moves_and_iterator_retain_one_charge_beyond_child_lease() {
    let _serial = SERIAL.lock().unwrap();
    #[repr(align(64))]
    struct Item {
        id: u8,
        nested: Box<[u8]>,
    }
    let quote = u64::try_from(Layout::array::<Item>(3).unwrap().size()).unwrap();
    let parent = authority().request_lease(request(quote)).unwrap();
    let (retained, pointer, nested_pointer) = {
        let child = parent.child(request(quote)).unwrap();
        let mut builder =
            ExecutionArrayBuilder::<Item>::allocate(3, Policy::Execution(&child)).unwrap();
        assert_eq!(builder.element_count(), 3);
        for id in 1..=3 {
            builder
                .push(Item {
                    id,
                    nested: vec![id].into_boxed_slice(),
                })
                .unwrap();
        }
        let pointer = builder.elements().as_ptr();
        let nested_pointer = builder.elements()[0].nested.as_ptr();
        assert_eq!((pointer as usize) % 64, 0);
        let denied = builder
            .push(Item {
                id: 9,
                nested: vec![9].into_boxed_slice(),
            })
            .unwrap_err();
        assert_eq!(denied.kind(), Kind::WriteBeyondReserved);
        assert_eq!(denied.requested_payload_bytes(), Some(quote));
        assert_eq!(builder.len(), 3);
        let array = builder.seal().unwrap();
        assert_eq!(array.elements().as_ptr(), pointer);
        assert_eq!(array[0].nested.as_ptr(), nested_pointer);
        (Arc::new(array), pointer, nested_pointer)
    };
    assert_eq!(retained.charged_payload_bytes(), Some(quote));
    assert!(matches!(
        parent.reserve_memory(1),
        Err(MemoryLimitDenial {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    let shared = Arc::clone(&retained);
    drop(retained);
    assert_eq!(shared.elements().as_ptr(), pointer);
    assert_eq!(shared[0].nested.as_ptr(), nested_pointer);
    let array = Arc::try_unwrap(shared).unwrap();
    assert_eq!(
        (&array).into_iter().map(|item| item.id).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let mut consuming = array.into_iter();
    assert_eq!(consuming.elements().as_ptr(), pointer);
    assert_eq!(consuming.size_hint(), (3, Some(3)));
    let first = consuming.next().unwrap();
    assert_eq!(first.id, 1);
    assert_eq!(first.nested.as_ptr(), nested_pointer);
    assert_eq!(consuming.next_back().unwrap().id, 3);
    assert_eq!(consuming.next().unwrap().id, 2);
    assert!(consuming.next().is_none());
    assert_eq!(consuming.charged_payload_bytes(), Some(quote));
    assert!(matches!(
        parent.reserve_memory(1),
        Err(MemoryLimitDenial {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    drop(consuming);
    assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
    // The separately owned nested allocation survives the array's backing.
    assert_eq!(first.nested.as_ref(), [1]);
}

#[test]
fn element_destruction_keeps_charge_until_partial_and_consuming_backings_drop() {
    let _serial = SERIAL.lock().unwrap();
    struct Tracked<'scope, 'authority> {
        parent: &'scope ExecutionResourceLease<'authority>,
        drops: &'scope Cell<usize>,
        charged_drops: &'scope Cell<usize>,
    }
    impl Drop for Tracked<'_, '_> {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
            if matches!(
                self.parent.reserve_memory(1),
                Err(MemoryLimitDenial {
                    requested: 1,
                    admitted: 0,
                    level: MemoryLimitLevel::Policy { ancestor: 0 }
                })
            ) {
                self.charged_drops.set(self.charged_drops.get() + 1);
            }
        }
    }
    let quote = u64::try_from(Layout::array::<Tracked<'_, '_>>(3).unwrap().size()).unwrap();
    let parent = authority().request_lease(request(quote)).unwrap();
    let drops = Cell::new(0);
    let charged_drops = Cell::new(0);
    let mut builder = ExecutionArrayBuilder::allocate(3, Policy::Execution(&parent)).unwrap();
    for _ in 0..3 {
        builder
            .push(Tracked {
                parent: &parent,
                drops: &drops,
                charged_drops: &charged_drops,
            })
            .unwrap();
    }
    let mut consuming = builder.seal().unwrap().into_iter();
    drop(consuming.next().unwrap());
    assert_eq!(drops.get(), 1);
    drop(consuming);
    assert_eq!(drops.get(), 3);
    assert_eq!(charged_drops.get(), 3);
    assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
    let mut partial = ExecutionArrayBuilder::allocate(3, Policy::Execution(&parent)).unwrap();
    partial
        .push(Tracked {
            parent: &parent,
            drops: &drops,
            charged_drops: &charged_drops,
        })
        .unwrap();
    let denied = partial.seal().unwrap_err();
    assert_eq!(denied.kind(), Kind::IncompleteSeal);
    assert_eq!(denied.requested_payload_bytes(), Some(quote));
    assert_eq!(drops.get(), 4); // Uninitialized tail is not dropped.
    assert_eq!(charged_drops.get(), 4);
    assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
}

#[test]
fn zst_logical_fill_and_zero_charge_do_not_use_allocator_capacity_as_count() {
    let _serial = SERIAL.lock().unwrap();
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    #[repr(align(64))]
    struct Zero;
    impl Drop for Zero {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
    let parent = authority().request_lease(request(0)).unwrap();
    let before = DROPS.load(Ordering::SeqCst);
    let mut builder =
        ExecutionArrayBuilder::<Zero>::allocate(3, Policy::Execution(&parent)).unwrap();
    for _ in 0..3 {
        builder.push(Zero).unwrap();
    }
    let denied = builder.push(Zero).unwrap_err();
    assert_eq!(denied.kind(), Kind::WriteBeyondReserved);
    assert_eq!(denied.requested_payload_bytes(), Some(0));
    assert_eq!(builder.len(), 3);
    assert_eq!(builder.element_count(), 3);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
    let array = builder.seal().unwrap();
    assert_eq!(array.charged_payload_bytes(), Some(0));
    let mut consuming = array.into_iter();
    assert_eq!(consuming.len(), 3);
    assert_eq!((consuming.elements().as_ptr() as usize) % 64, 0);
    drop(consuming.next().unwrap());
    assert_eq!(consuming.size_hint(), (2, Some(2)));
    assert_eq!((consuming.elements().as_ptr() as usize) % 64, 0);
    drop(consuming.next_back().unwrap());
    assert_eq!(consuming.len(), 1);
    assert_eq!(consuming.charged_payload_bytes(), Some(0));
    drop(consuming);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 4);
    let leased_empty = ExecutionArrayBuilder::<u64>::allocate(0, Policy::Execution(&parent))
        .unwrap()
        .seal()
        .unwrap();
    let system_empty = ExecutionArrayBuilder::<u64>::allocate(0, Policy::SystemAllocation)
        .unwrap()
        .seal()
        .unwrap();
    assert!(leased_empty.is_empty());
    assert_eq!(leased_empty.charged_payload_bytes(), Some(0));
    assert_eq!(system_empty.charged_payload_bytes(), None);
    assert_eq!(leased_empty, system_empty);
}

#[test]
fn typed_layout_and_live_denials_preserve_quotes_and_release_partial_storage() {
    let _serial = SERIAL.lock().unwrap();
    let denied =
        ExecutionArrayBuilder::<u64>::allocate(usize::MAX, Policy::SystemAllocation).unwrap_err();
    assert_eq!(denied.kind(), Kind::Layout);
    assert_eq!(denied.requested_payload_bytes(), None);
    let small = authority().request_lease(request(7)).unwrap();
    let denied = ExecutionArrayBuilder::<u32>::allocate(2, Policy::Execution(&small)).unwrap_err();
    assert_eq!(
        denied.kind(),
        Kind::Lease(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
            requested: 8,
            admitted: 7,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        }))
    );
    assert_eq!(denied.requested_payload_bytes(), Some(8));
    assert_eq!(small.reserve_memory(7).unwrap().bytes(), 7);
    let parent = authority().request_lease(request(16)).unwrap();
    let mut stopped_request = request(16);
    let stop = CancellationSource::new();
    stopped_request.cancellation = stop.token();
    let child = parent.child(stopped_request.clone()).unwrap();
    let policy = Policy::Execution(&child);
    let mut builder = ExecutionArrayBuilder::<u32>::allocate(4, policy).unwrap();
    builder.push(1).unwrap();
    stop.cancel();
    assert_eq!(
        policy.check_live().unwrap_err().requested_payload_bytes(),
        None
    );
    let denied = builder.push(2).unwrap_err();
    assert_eq!(denied.kind(), Kind::Cancelled);
    assert_eq!(denied.requested_payload_bytes(), Some(16));
    assert_eq!(builder.elements(), [1]);
    assert_eq!(builder.seal().unwrap_err(), denied);
    assert_eq!(parent.reserve_memory(16).unwrap().bytes(), 16);
    stopped_request.cancellation = CancellationToken::new();
    stopped_request.deadline = Some(Instant::now().checked_sub(Duration::from_secs(1)).unwrap());
    let expired = parent.child(stopped_request).unwrap();
    let denied =
        ExecutionArrayBuilder::<u32>::allocate(4, Policy::Execution(&expired)).unwrap_err();
    assert_eq!(denied.kind(), Kind::DeadlineElapsed);
    assert_eq!(denied.requested_payload_bytes(), Some(16));
    assert_eq!(parent.reserve_memory(16).unwrap().bytes(), 16);
}

#[test]
fn an_unbounded_process_still_refuses_counter_overflow_before_charging() {
    let _serial = SERIAL.lock().unwrap();
    let first = authority().request_lease(request(u64::MAX)).unwrap();
    let second = authority().request_lease(request(u64::MAX)).unwrap();
    let held = first.reserve_memory(u64::MAX - 1).unwrap();
    assert_eq!(
        second.reserve_memory(2).unwrap_err(),
        MemoryLimitDenial {
            requested: 2,
            admitted: 1,
            level: MemoryLimitLevel::Process,
        }
    );
    let last = second.reserve_memory(1).unwrap();
    assert_eq!(held.bytes(), u64::MAX - 1);
    drop(last);
    drop(held);
    assert_eq!(second.reserve_memory(u64::MAX).unwrap().bytes(), u64::MAX);
}
