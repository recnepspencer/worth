use super::*;
use crate::data::persistent_ord_map::PersistentOrdMap;
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial as Denial,
};

#[test]
fn shared_index_history_prepares_without_copying_and_mutation_invalidates_only_its_fork() {
    let mut history = DiagnosticHistory::new();
    history.push_back("retained payload".repeat(128)).unwrap();
    let mut index = PersistentOrdMap::new();
    index.insert(7_u64, history);
    let mut fork = index.fork_persistent();
    let source = index.get(&7).unwrap();
    let sibling = fork.get(&7).unwrap();
    assert!(std::ptr::eq(source, sibling));
    let entries = source.entries.clone();
    let wire = serde_json::to_value(source).unwrap();
    let mut measured_work = Work::new(1000);
    let expected = source.retained_heap_charge(&mut measured_work).unwrap();
    let limit = measured_work.visits() - 1;
    assert_eq!(
        source.prepare_retained_charge(&mut Work::new(limit)),
        Err(Denial::WorkExhausted {
            maximum_visits: limit
        })
    );
    assert_eq!(
        source.prepared_retained_charge(),
        Err(DiagnosticHistoryEditDenial::PreparationRequired)
    );
    assert_eq!(
        source
            .prepare_retained_charge(&mut Work::new(measured_work.visits()))
            .unwrap(),
        expected
    );
    assert_eq!(sibling.prepared_retained_charge().unwrap(), expected);
    assert!(index.ptr_eq(&fork));
    assert!(source.entries.ptr_eq(&entries));
    assert_eq!(serde_json::to_value(source).unwrap(), wire);
    fork.get_mut(&7)
        .unwrap()
        .push_back("new payload".into())
        .unwrap();
    assert_eq!(
        fork.get(&7).unwrap().prepared_retained_charge(),
        Err(DiagnosticHistoryEditDenial::PreparationRequired)
    );
    assert_eq!(
        index.get(&7).unwrap().prepared_retained_charge().unwrap(),
        expected
    );
    assert_eq!(serde_json::to_value(index.get(&7).unwrap()).unwrap(), wire);
    assert!(index.get(&7).unwrap().entries.ptr_eq(&entries));
}

#[test]
fn concurrent_cold_history_preparation_converges_on_one_nonsemantic_fact() {
    let mut history = DiagnosticHistory::new();
    history.push_back("shared evidence".repeat(64)).unwrap();
    let expected = history.retained_heap_charge(&mut Work::new(1000)).unwrap();
    std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            history
                .prepare_retained_charge(&mut Work::new(1000))
                .unwrap()
        });
        let right = scope.spawn(|| {
            history
                .prepare_retained_charge(&mut Work::new(1000))
                .unwrap()
        });
        assert_eq!(left.join().unwrap(), expected);
        assert_eq!(right.join().unwrap(), expected);
    });
    assert_eq!(history.prepared_retained_charge().unwrap(), expected);
}
