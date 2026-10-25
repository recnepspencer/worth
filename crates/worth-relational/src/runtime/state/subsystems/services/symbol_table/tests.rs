use super::RuntimeSymbolTable;

#[test]
fn cancellation_after_backing_admission_refuses_configuration_publication() {
    use crate::symbols::data::{RelationalFieldTextAdmissionDenial as Denial, StoredSymbolText};
    use worth_execution::{CancellationSource, ExecutionImmutableBytes, ExecutionRequest};

    let runtime = crate::tests::support::runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let head = runtime.observe_branch(&identity).unwrap().0;
    let owner = &runtime.services.symbols;
    let before = owner.interner_snapshot();
    let configuration_before = owner.configuration_snapshot();
    let cancellation = CancellationSource::new();
    let request =
        crate::test_execution_policy::serial_request().with_cancellation(cancellation.token());
    let source = ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(
        b"late-cancelled-text".to_vec().into_boxed_slice(),
    ));
    let configuration = owner.configuration_snapshot.write().unwrap();
    let (admitted, received) = std::sync::mpsc::sync_channel(0);
    let result = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let execution = ExecutionRequest::serial(&request);
            let mut state = owner.state.write().unwrap();
            state.check_text_capacity("late-cancelled-text").unwrap();
            let text = StoredSymbolText::admit(&execution, &source).unwrap();
            // Admission has completed; the actual publication lock is still held.
            admitted.send(()).unwrap();
            owner.publish_admitted_text(&execution, &mut state, text)
        });
        received.recv().unwrap();
        cancellation.cancel();
        drop(configuration);
        worker.join().unwrap()
    });
    assert!(matches!(
        result,
        Err(Denial::Stopped(worth_execution::MapKernelStop::Cancelled))
    ));
    assert_eq!(owner.interner_snapshot(), before);
    assert_eq!(owner.configuration_snapshot(), configuration_before);
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, head);
    assert_eq!(source.bytes(), b"late-cancelled-text");
}

#[test]
fn configuration_snapshot_stays_sorted_and_detached_owners_do_not_share_new_keys() {
    let owner = RuntimeSymbolTable::default();
    owner.normalize_client_keys(|interner| {
        let beta = interner.intern("beta");
        let alpha = interner.intern("alpha");
        vec![(beta, "beta".to_string()), (alpha, "alpha".to_string())]
    });
    let detached = owner.detached_owner_snapshot();
    owner.normalize_client_keys(|interner| {
        let gamma = interner.intern("gamma");
        vec![(gamma, "gamma".to_string())]
    });

    let values = |table: &RuntimeSymbolTable| {
        table
            .configuration_snapshot()
            .entries
            .into_iter()
            .map(|(_, value)| value)
            .collect::<Vec<_>>()
    };
    assert_eq!(values(&owner), ["alpha", "beta", "gamma"]);
    assert_eq!(values(&detached), ["alpha", "beta"]);
}
