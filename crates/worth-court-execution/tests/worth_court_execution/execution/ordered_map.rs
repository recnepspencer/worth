//! Public native map construction, independent ordered truth and original custody.
use std::collections::BTreeMap;
use worth_execution::{
    CancellationSource, ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest,
    MapKernelStop, SerialMemoryBudget, SerialRequest, WorkCeilingDenial,
};

fn backing(bytes: u64, cancel: &CancellationSource) -> SerialRequest {
    SerialRequest::from_memory(SerialMemoryBudget::new(Some(bytes)), cancel.token(), None)
}
fn cell_bytes() -> u64 {
    let cancel = CancellationSource::new();
    let storage = backing(4096, &cancel);
    let map = ExecutionOrderedMap::from_sorted_native_for_request(
        1,
        [(0u64, 0u64)],
        ExecutionRequest::serial(&storage),
    )
    .unwrap()
    .unwrap();
    let bytes = storage.memory().unwrap().charged_memory_bytes();
    assert!(bytes > size_of::<(u64, u64)>() as u64);
    assert_eq!(map.retained_bytes(), bytes);
    drop(map);
    assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    bytes
}
fn verify(map: &ExecutionOrderedMap<u64, u64>, truth: &BTreeMap<u64, u64>) {
    assert_eq!(map.len(), truth.len());
    assert!(map.iter().eq(truth.iter()));
    for (rank, (key, value)) in truth.iter().enumerate() {
        assert_eq!(map.rank_of(key), Some(rank));
        assert_eq!(map.entry_at(rank), Some((key, value)));
        assert_eq!(map.get(key), Some(value));
    }
    assert_eq!(map.entry_at(truth.len()), None);
}

#[test]
fn sorted_native_cells_match_independent_order_and_remain_editable() {
    for count in [0, 1, 2, 3, 31, 32, 33, 257] {
        let cancel = CancellationSource::new();
        let storage = backing(2_000_000, &cancel);
        let request = ExecutionRequest::serial(&storage);
        let mut truth: BTreeMap<_, _> = (0..count).map(|n| (n * 3, n + 100)).collect();
        let mut map = ExecutionOrderedMap::from_sorted_native_for_request(
            truth.len(),
            truth.iter().map(|(&k, &v)| (k, v)),
            request,
        )
        .unwrap()
        .unwrap();
        verify(&map, &truth);
        assert_eq!(
            storage.memory().unwrap().charged_memory_bytes(),
            count * cell_bytes()
        );
        assert_eq!(
            map.retained_bytes(),
            storage.memory().unwrap().charged_memory_bytes()
        );
        let old = map.clone();
        let old_truth = truth.clone();
        map.insert_cloned_native_for_request(&1, &999, request)
            .unwrap();
        truth.insert(1, 999);
        for key in old_truth.keys().step_by(3) {
            map.remove_for_request(key, request).unwrap();
            truth.remove(key);
            verify(&map, &truth);
        }
        verify(&old, &old_truth);
        verify(&map, &truth);
        drop(map);
        drop(old);
        assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    }
}

#[test]
fn sorted_native_input_requires_exact_length_and_strict_keys() {
    let cancel = CancellationSource::new();
    let storage = backing(2_000_000, &cancel);
    let request = ExecutionRequest::serial(&storage);
    for (count, input) in [
        (0, vec![(0u64, 1u64)]),
        (2, vec![(0, 1)]),
        (1, vec![(0, 1), (1, 2)]),
        (2, vec![(1, 1), (1, 2)]),
        (2, vec![(2, 1), (1, 2)]),
    ] {
        assert!(
            ExecutionOrderedMap::from_sorted_native_for_request(count, input, request)
                .unwrap()
                .is_none()
        );
        assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    }
}

#[test]
fn sorted_native_refusals_release_every_partial_cell_and_fresh_retry_works() {
    let required = 7 * cell_bytes();
    for bytes in (0..required).step_by(16) {
        let cancel = CancellationSource::new();
        let storage = backing(bytes, &cancel);
        let result = ExecutionOrderedMap::from_sorted_native_for_request(
            7,
            (0..7u64).map(|n| (n, n)),
            ExecutionRequest::serial(&storage),
        );
        assert!(
            matches!(
                result,
                Err(ExecutionOrderedMapDenial::Request(
                    WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
                ))
            ),
            "bytes={bytes}, result={result:?}"
        );
        assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    }
    for stop in [0, 3, 6] {
        let cancel = CancellationSource::new();
        let storage = backing(required + 4096, &cancel);
        let input = (0..7u64)
            .inspect(|n| {
                if *n == stop {
                    cancel.cancel();
                }
            })
            .map(|n| (n, n));
        let result = ExecutionOrderedMap::from_sorted_native_for_request(
            7,
            input,
            ExecutionRequest::serial(&storage),
        );
        assert!(
            matches!(
                result,
                Err(ExecutionOrderedMapDenial::Request(
                    WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
                ))
            ),
            "{result:?}"
        );
        assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    }
    let cancel = CancellationSource::new();
    let storage = backing(required + 4096, &cancel);
    let map = ExecutionOrderedMap::from_sorted_native_for_request(
        7,
        (0..7u64).map(|n| (n, n)),
        ExecutionRequest::serial(&storage),
    )
    .unwrap()
    .unwrap();
    assert_eq!(map.len(), 7);
    drop(map);
    assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn sorted_native_final_stop_dominates_invalid_input() {
    let cancel = CancellationSource::new();
    let storage = backing(10_000, &cancel);
    let input = std::iter::once((0u64, 0u64)).inspect(|_| {
        cancel.cancel();
    });
    let result = ExecutionOrderedMap::from_sorted_native_for_request(
        0,
        input,
        ExecutionRequest::serial(&storage),
    );
    assert!(
        matches!(
            result,
            Err(ExecutionOrderedMapDenial::Request(
                WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
            ))
        ),
        "{result:?}"
    );
    assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
}
