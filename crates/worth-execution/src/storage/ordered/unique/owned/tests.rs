use super::*;
use crate::{CancellationSource, SerialMemoryBudget, SerialRequest};
use std::sync::atomic::{AtomicUsize, Ordering};

// Deliberately neither Copy nor Clone. Its input signal already exists before
// SDK admission; this test claims only the actual new Entry and Node controls.
struct Signal(Arc<AtomicUsize>);
impl Drop for Signal {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn bill() -> u64 {
    let arc = |layout: std::alloc::Layout| {
        std::alloc::Layout::new::<[usize; 2]>()
            .extend(layout)
            .unwrap()
            .0
            .pad_to_align()
            .size() as u64
    };
    arc(std::alloc::Layout::new::<node::Entry<u64, Signal>>())
        + arc(std::alloc::Layout::new::<node::Node<u64, Signal>>())
}

#[test]
fn owned_nonclone_payload_moves_through_unique_cells_and_final_retirement() {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let signal = Arc::new(AtomicUsize::new(0));
    let mut map = ExecutionOrderedMap::<u64, Signal>::default();
    for key in [4, 2, 6, 1, 3, 5, 7] {
        let cell = ExecutionOrderedMapCell::prepare_owned_native_for_request(
            key,
            Signal(signal.clone()),
            ExecutionRequest::serial(&host),
        )
        .unwrap();
        assert!(map.insert_owned_vacant_prepared(cell).is_ok());
    }
    assert_eq!(budget.charged_memory_bytes(), 7 * bill());
    let removed = map.remove_owned_retaining_unique(&4).unwrap().unwrap();
    assert_eq!(*removed.key(), 4);
    assert_eq!(signal.load(Ordering::SeqCst), 0);
    assert_eq!(budget.charged_memory_bytes(), 7 * bill());
    assert!(map.insert_owned_vacant_prepared(removed).is_ok());
    let removed = map.remove_owned_retaining_unique(&2).unwrap().unwrap();
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [1, 3, 4, 5, 6, 7]);
    drop(map);
    assert_eq!(signal.load(Ordering::SeqCst), 6);
    assert_eq!(budget.charged_memory_bytes(), bill());
    drop(removed);
    assert_eq!(signal.load(Ordering::SeqCst), 7);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn owned_unique_refusal_returns_same_payload_and_preserves_shared_original() {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let signal = Arc::new(AtomicUsize::new(0));
    let request = ExecutionRequest::serial(&host);
    let mut map = ExecutionOrderedMap::<u64, Signal>::default();
    assert!(map
        .insert_owned_vacant_prepared(
            ExecutionOrderedMapCell::prepare_owned_native_for_request(
                1,
                Signal(signal.clone()),
                request
            )
            .unwrap()
        )
        .is_ok());
    let alias = map.clone();
    let cell = ExecutionOrderedMapCell::prepare_owned_native_for_request(
        2,
        Signal(signal.clone()),
        request,
    )
    .unwrap();
    let original = cell.value() as *const Signal;
    let (cause, cell) = map.insert_owned_vacant_prepared(cell).err().unwrap();
    assert_eq!(cause, ExecutionOrderedMapUniqueDenial::SharedStorage);
    assert_eq!(cell.value() as *const Signal, original);
    assert!(matches!(
        map.remove_owned_retaining_unique(&1),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    ));
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [1]);
    assert_eq!(budget.charged_memory_bytes(), 2 * bill());
    drop(alias);
    assert!(map.insert_owned_vacant_prepared(cell).is_ok());
    drop(map);
    assert_eq!(signal.load(Ordering::SeqCst), 2);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
