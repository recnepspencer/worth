// Physical typed payload custody; nested payload heaps remain uncharged here.
#[allow(unused_imports)]
use crate::array_backing::isolated_context::*;
#[allow(unused_imports)]
pub(crate) use crate::test_serialization;

use {std::cell::Cell, std::sync::atomic::AtomicUsize};
use {worth_execution::ExecutionArrayBuilder, worth_execution::ExecutionResourceLease};

#[test]
fn typed_layout_moves_and_iterator_retain_one_charge_beyond_child_lease() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = crate::test_serialization::guard(&SERIAL);
    #[repr(align(64))]
    struct Item {
        id: u8,
        nested: Box<[u8]>,
    }
    let quote = u64::try_from(std::alloc::Layout::array::<Item>(3).unwrap().size()).unwrap();
    let parent = authority().request_lease(request(quote)).unwrap();
    let (retained, pointer, nested_pointer) = {
        let child = parent.child(request(quote)).unwrap();
        let mut builder = ExecutionArrayBuilder::<Item>::allocate(
            3,
            worth_execution::ExecutionAllocationPolicy::Execution(&child),
        )
        .unwrap();
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
        assert_eq!(
            denied.classify(),
            worth_execution::ExecutionAllocationDenialClass::Physical(
                worth_execution::ExecutionPhysicalAllocationDenialKind::WriteBeyondReserved
            )
        );
        assert_eq!(denied.requested_payload_bytes(), Some(quote));
        assert_eq!(builder.len(), 3);
        let array = builder.seal().unwrap();
        assert_eq!(array.elements().as_ptr(), pointer);
        assert_eq!(array[0].nested.as_ptr(), nested_pointer);
        (std::sync::Arc::new(array), pointer, nested_pointer)
    };
    assert_eq!(retained.charged_payload_bytes(), Some(quote));
    assert!(matches!(
        parent.reserve_memory(1),
        Err(worth_execution::MemoryLimitDenial::Capacity {
            requested: 1,
            admitted: 0,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    let shared = std::sync::Arc::clone(&retained);
    drop(retained);
    assert_eq!(shared.elements().as_ptr(), pointer);
    assert_eq!(shared[0].nested.as_ptr(), nested_pointer);
    let array = std::sync::Arc::try_unwrap(shared).unwrap();
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
        Err(worth_execution::MemoryLimitDenial::Capacity {
            requested: 1,
            admitted: 0,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    drop(consuming);
    assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
    // The separately owned nested allocation survives the array's backing.
    assert_eq!(first.nested.as_ref(), [1]);
}

#[test]
fn element_destruction_keeps_charge_until_partial_and_consuming_backings_drop() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = crate::test_serialization::guard(&SERIAL);
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
                Err(worth_execution::MemoryLimitDenial::Capacity {
                    requested: 1,
                    admitted: 0,
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
                })
            ) {
                self.charged_drops.set(self.charged_drops.get() + 1);
            }
        }
    }
    let quote = u64::try_from(
        std::alloc::Layout::array::<Tracked<'_, '_>>(3)
            .unwrap()
            .size(),
    )
    .unwrap();
    let parent = authority().request_lease(request(quote)).unwrap();
    let drops = std::cell::Cell::new(0);
    let charged_drops = std::cell::Cell::new(0);
    let mut builder = worth_execution::ExecutionArrayBuilder::allocate(
        3,
        worth_execution::ExecutionAllocationPolicy::Execution(&parent),
    )
    .unwrap();
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
    let mut partial = worth_execution::ExecutionArrayBuilder::allocate(
        3,
        worth_execution::ExecutionAllocationPolicy::Execution(&parent),
    )
    .unwrap();
    partial
        .push(Tracked {
            parent: &parent,
            drops: &drops,
            charged_drops: &charged_drops,
        })
        .unwrap();
    let denied = partial.seal().unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Physical(
            worth_execution::ExecutionPhysicalAllocationDenialKind::IncompleteSeal
        )
    );
    assert_eq!(denied.requested_payload_bytes(), Some(quote));
    assert_eq!(drops.get(), 4); // Uninitialized tail is not dropped.
    assert_eq!(charged_drops.get(), 4);
    assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
}

#[test]
fn zst_logical_fill_and_zero_charge_do_not_use_allocator_capacity_as_count() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = crate::test_serialization::guard(&SERIAL);
    static DROPS: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    #[repr(align(64))]
    struct Zero;
    impl Drop for Zero {
        fn drop(&mut self) {
            DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let parent = authority().request_lease(request(0)).unwrap();
    let before = DROPS.load(std::sync::atomic::Ordering::SeqCst);
    let mut builder = ExecutionArrayBuilder::<Zero>::allocate(
        3,
        worth_execution::ExecutionAllocationPolicy::Execution(&parent),
    )
    .unwrap();
    for _ in 0..3 {
        builder.push(Zero).unwrap();
    }
    let denied = builder.push(Zero).unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Physical(
            worth_execution::ExecutionPhysicalAllocationDenialKind::WriteBeyondReserved
        )
    );
    assert_eq!(denied.requested_payload_bytes(), Some(0));
    assert_eq!(builder.len(), 3);
    assert_eq!(builder.element_count(), 3);
    assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst), before + 1);
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
    assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst), before + 4);
    let leased_empty = ExecutionArrayBuilder::<u64>::allocate(
        0,
        worth_execution::ExecutionAllocationPolicy::Execution(&parent),
    )
    .unwrap()
    .seal()
    .unwrap();
    let system_empty = ExecutionArrayBuilder::<u64>::allocate(
        0,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
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
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = crate::test_serialization::guard(&SERIAL);
    let denied = ExecutionArrayBuilder::<u64>::allocate(
        usize::MAX,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Physical(
            worth_execution::ExecutionPhysicalAllocationDenialKind::Layout
        )
    );
    assert_eq!(denied.requested_payload_bytes(), None);
    let small = authority().request_lease(request(7)).unwrap();
    let denied = ExecutionArrayBuilder::<u32>::allocate(
        2,
        worth_execution::ExecutionAllocationPolicy::Execution(&small),
    )
    .unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Request(
            worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(
                    worth_execution::MemoryLimitDenial::Capacity {
                        requested: 8,
                        admitted: 7,
                        level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
                    }
                )
            )
        )
    );
    assert_eq!(denied.requested_payload_bytes(), Some(8));
    assert_eq!(small.reserve_memory(7).unwrap().bytes(), 7);
    let parent = authority().request_lease(request(16)).unwrap();
    let mut stopped_request = request(16);
    let stop = worth_execution::CancellationSource::new();
    stopped_request.cancellation = stop.token();
    let child = parent.child(stopped_request.clone()).unwrap();
    let policy = worth_execution::ExecutionAllocationPolicy::Execution(&child);
    let mut builder = ExecutionArrayBuilder::<u32>::allocate(4, policy).unwrap();
    builder.push(1).unwrap();
    stop.cancel();
    assert_eq!(
        policy.check_live().unwrap_err().requested_payload_bytes(),
        None
    );
    let denied = builder.push(2).unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Request(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        )
    );
    assert_eq!(denied.requested_payload_bytes(), Some(16));
    assert_eq!(builder.elements(), [1]);
    assert_eq!(builder.seal().unwrap_err(), denied);
    assert_eq!(parent.reserve_memory(16).unwrap().bytes(), 16);
    stopped_request.cancellation = worth_execution::CancellationToken::new();
    stopped_request.deadline = Some(
        std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap(),
    );
    let expired = parent.child(stopped_request).unwrap();
    let denied = ExecutionArrayBuilder::<u32>::allocate(
        4,
        worth_execution::ExecutionAllocationPolicy::Execution(&expired),
    )
    .unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Request(
            worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::DeadlineElapsed
            )
        )
    );
    assert_eq!(denied.requested_payload_bytes(), Some(16));
    assert_eq!(parent.reserve_memory(16).unwrap().bytes(), 16);
}

#[test]
fn an_unbounded_process_still_refuses_counter_overflow_before_charging() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = crate::test_serialization::guard(&SERIAL);
    let first = authority().request_lease(request(u64::MAX)).unwrap();
    let second = authority().request_lease(request(u64::MAX)).unwrap();
    let held = first.reserve_memory(u64::MAX - 1).unwrap();
    assert_eq!(
        second.reserve_memory(2).unwrap_err(),
        worth_execution::MemoryLimitDenial::CounterOverflow {
            requested: 2,
            charged: u64::MAX - 1,
            level: worth_execution::MemoryLimitLevel::Process,
        }
    );
    let last = second.reserve_memory(1).unwrap();
    assert_eq!(held.bytes(), u64::MAX - 1);
    drop(last);
    drop(held);
    assert_eq!(second.reserve_memory(u64::MAX).unwrap().bytes(), u64::MAX);
}
