use super::*;

#[test]
fn publication_excludes_abandoned_preview_meanings_adopted_for_reading() {
    let owner = RuntimeSymbolTable::default();
    let mut preview = owner.begin_pending_meanings();
    let unused = preview.reserve("rolled-back-preview").unwrap();
    let mut candidate = owner.begin_pending_meanings();
    candidate.adopt(&preview);
    let used = candidate
        .reserve_revision_reference("actual-field")
        .unwrap();
    let meanings = SymbolTableSnapshot {
        entries: vec![(used, "actual-field".into())],
    };
    let publication = candidate.seal_for_publication(&meanings);
    drop(preview);
    publication.publish().unwrap();
    assert_eq!(
        owner.with_read(|symbols| symbols.resolve(unused).map(str::to_owned)),
        None
    );
    assert_eq!(
        owner.with_read(|symbols| symbols.resolve(used).map(str::to_owned)),
        Some("actual-field".to_owned())
    );
    assert!(owner.reservations.lock().unwrap().pending.is_empty());
}

#[test]
fn concurrent_pending_meanings_deduplicate_and_publish_only_the_accepted_candidate() {
    let owner = RuntimeSymbolTable::default();
    let mut abandoned = owner.begin_pending_meanings();
    let mut accepted = owner.begin_pending_meanings();
    let shared = abandoned.reserve("shared").unwrap();
    assert_eq!(accepted.reserve("shared").unwrap(), shared);
    let unused = abandoned.reserve("abandoned-only").unwrap();
    assert!(owner.configuration_snapshot().entries.is_empty());
    assert_eq!(owner.with_read(|symbols| symbols.symbol("shared")), None);
    assert_eq!(
        accepted
            .binding()
            .with_read(|symbols| symbols.resolve(shared).map(str::to_owned)),
        Some("shared".to_owned())
    );
    drop(abandoned);
    let meanings = accepted.interner().snapshot();
    accepted.seal_for_publication(&meanings).publish().unwrap();
    assert_eq!(
        owner.with_read(|symbols| symbols.symbol("shared")),
        Some(shared)
    );
    assert_eq!(
        owner.with_read(|symbols| symbols.resolve(unused).map(str::to_owned)),
        None
    );
    assert!(owner.reservations.lock().unwrap().pending.is_empty());
}

#[test]
fn abandoning_the_last_custodian_spends_the_identity_and_removes_pending_text() {
    let owner = RuntimeSymbolTable::default();
    let mut first = owner.begin_pending_meanings();
    let abandoned = first.reserve("retry").unwrap();
    drop(first);
    assert!(owner.reservations.lock().unwrap().pending.is_empty());
    let mut retry = owner.begin_pending_meanings();
    let fresh = retry.reserve("retry").unwrap();
    assert!(fresh.0 > abandoned.0);
    let meanings = retry.interner().snapshot();
    retry.seal_for_publication(&meanings).publish().unwrap();
    assert_eq!(
        owner.with_read(|symbols| symbols.symbol("retry")),
        Some(fresh)
    );
    assert_eq!(
        owner.with_read(|symbols| symbols.resolve(abandoned).map(str::to_owned)),
        None
    );
}

#[test]
fn independent_admission_uses_the_pending_identity() {
    let owner = RuntimeSymbolTable::default();
    let mut candidate = owner.begin_pending_meanings();
    let symbol = candidate.reserve("independent").unwrap();
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let bytes = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
        b"independent".to_vec().into_boxed_slice(),
    ));
    assert_eq!(
        owner.admit_field_text(&execution, &bytes).unwrap().0,
        symbol
    );
    let meanings = candidate.interner().snapshot();
    candidate.seal_for_publication(&meanings).publish().unwrap();
    assert_eq!(
        owner.with_read(|symbols| symbols.symbol("independent")),
        Some(symbol)
    );
}

#[test]
fn parallel_candidates_share_pending_meaning_without_publishing_it() {
    let owner = RuntimeSymbolTable::default();
    let barrier = std::sync::Barrier::new(8);
    let candidates = std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let mut candidate = owner.begin_pending_meanings();
                    barrier.wait();
                    let symbol = candidate.reserve("parallel").unwrap();
                    (candidate, symbol)
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(candidates
        .iter()
        .all(|(_, symbol)| *symbol == candidates[0].1));
    assert!(owner.snapshot().entries.is_empty());
    drop(candidates);
    assert!(owner.reservations.lock().unwrap().pending.is_empty());
}

#[test]
fn checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers() {
    use crate::tests::support::*;
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(&execution, &runtime, "symbol-frontier-seed");
    let mut abandoned = runtime.services.symbols.begin_pending_meanings();
    let symbol = abandoned.reserve("abandoned-before-checkpoint").unwrap();
    drop(abandoned);
    let (_, recovered) =
        checkpoint_and_recover_with(&execution, &runtime, durable_runtime_with_test_schema);
    let mut candidate = recovered.services.symbols.begin_pending_meanings();
    let fresh = loop {
        match candidate.reserve("abandoned-before-checkpoint") {
            Ok(symbol) => break symbol,
            Err(SymbolReservationDenial::ColdBucket(text)) => {
                drop(candidate);
                recovered
                    .services
                    .symbols
                    .demand_text(&execution, text.as_str())
                    .unwrap();
                candidate = recovered.services.symbols.begin_pending_meanings();
            }
            other => panic!("recovered symbol reservation failed: {other:?}"),
        }
    };
    assert!(fresh.0 > symbol.0);
    assert_eq!(
        recovered
            .services
            .symbols
            .with_read(|symbols| symbols.resolve(symbol).map(str::to_owned)),
        None
    );
    let meanings = candidate.interner().snapshot();
    candidate.seal_for_publication(&meanings).publish().unwrap();
}
