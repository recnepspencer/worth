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
    let (admitted, received) = std::sync::mpsc::sync_channel(0);
    let (resume, proceed) = std::sync::mpsc::sync_channel(0);
    let result = std::thread::scope(|scope| {
        let request = &request;
        let source = &source;
        let worker = scope.spawn(move || {
            let execution = ExecutionRequest::serial(&request);
            let text = StoredSymbolText::admit(&execution, &source).unwrap();
            // Cancel after backing admission and before dictionary publication.
            admitted.send(()).unwrap();
            proceed.recv().unwrap();
            owner.update(|state| owner.publish_admitted_text(&execution, state, text))
        });
        received.recv().unwrap();
        cancellation.cancel();
        resume.send(()).unwrap();
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
        let beta = interner
            .intern("beta")
            .expect("symbol fixture has identity capacity");
        let alpha = interner
            .intern("alpha")
            .expect("symbol fixture has identity capacity");
        vec![(beta, "beta".to_string()), (alpha, "alpha".to_string())]
    });
    let detached = owner.detached_owner_snapshot();
    owner.normalize_client_keys(|interner| {
        let gamma = interner
            .intern("gamma")
            .expect("symbol fixture has identity capacity");
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

#[test]
fn interrupted_native_dictionary_edit_preserves_both_views() {
    let owner = RuntimeSymbolTable::default();
    owner.normalize_client_keys(|symbols| {
        vec![(symbols.intern("before").unwrap(), "before".into())]
    });
    let before = owner.interner_snapshot();
    let configuration = owner.configuration_snapshot();
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        owner.normalize_client_keys(|symbols| {
            symbols
                .intern("discarded")
                .expect("symbol fixture has identity capacity");
            panic!("interrupted native dictionary edit")
        });
    }));
    assert!(interrupted.is_err());
    assert_eq!(owner.interner_snapshot(), before);
    assert_eq!(owner.configuration_snapshot(), configuration);
    owner.normalize_client_keys(|symbols| vec![(symbols.intern("after").unwrap(), "after".into())]);
    assert!(owner.with_read(|symbols| symbols.contains("after")));
    assert!(!owner.with_read(|symbols| symbols.contains("discarded")));
}
