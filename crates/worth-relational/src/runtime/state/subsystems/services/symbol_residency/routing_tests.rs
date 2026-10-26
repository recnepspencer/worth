//! Real checkpoint pages must preserve bounded lookup and authenticated routing.
use super::*;
use sha2::{Digest, Sha256};

fn colliding_texts() -> Vec<String> {
    (0..)
        .map(|id| format!("same-symbol-bucket-{id}"))
        .filter(|text| Sha256::digest(text.as_bytes())[0] == 0)
        .take(602)
        .collect()
}

fn collision_dictionary() -> (
    RuntimeSymbolTable,
    RelationalCheckpointDirectory,
    Arc<PageSource>,
    Vec<(Symbol, String)>,
) {
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    let runtime = runtime_with_test_schema();
    create_entity(&execution, &runtime, "symbol-page-seed");
    let transaction = test_owner_begin_transaction_for_main(&runtime);
    let texts = colliding_texts()
        .into_iter()
        .take(601)
        .map(|text| {
            let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
                text.as_bytes().to_vec().into_boxed_slice(),
            ));
            let admitted = runtime
                .admit_field_text(&execution, &transaction.basis, &input)
                .unwrap();
            (admitted.symbol, text)
        })
        .collect();
    let (table, directory, source, _) = checkpoint_dictionary(&runtime);
    (table, directory, source, texts)
}

#[test]
fn captured_collision_bucket_is_paged_and_late_text_keeps_its_identity() {
    let (table, directory, source, texts) = collision_dictionary();
    let bucket = directory
        .manifest
        .symbol_buckets
        .iter()
        .find(|bucket| bucket.bucket == 0)
        .unwrap();
    assert!(
        (3..5).contains(&bucket.pages.len()),
        "collision population fits bounded page headroom"
    );
    assert_eq!(source.reads.load(Ordering::SeqCst), 0);
    assert!(table.snapshot().entries.is_empty());
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    for locator in &bucket.pages {
        let page = crate::durability::checkpoints::symbol_pages::readmit(
            &execution,
            &source.members[locator],
            *locator,
            SymbolPageSelection::HashBucket(0).subject(),
        )
        .unwrap();
        assert!(!page.entries.is_empty() && page.entries.len() <= 256);
    }
    let (symbol, text) = texts.last().unwrap();
    let frontier = table.interner_snapshot().identity_frontier();
    table.demand_text(&execution, text).unwrap();
    let mut pending = table.begin_pending_meanings();
    assert_eq!(pending.reserve(text).unwrap(), *symbol);
    assert_eq!(table.interner_snapshot().identity_frontier(), frontier);
    assert!(source.reads.load(Ordering::SeqCst) < 5);
    assert!(
        table
            .snapshot()
            .entries
            .iter()
            .all(|(_, text)| Sha256::digest(text.as_str().as_bytes())[0] == 0),
        "unrelated symbol buckets stay cold"
    );
}

#[test]
fn a_late_bucket_fault_cannot_issue_a_new_spelling_and_stays_damaged() {
    let (table, directory, source, _) = collision_dictionary();
    let bucket = directory
        .manifest
        .symbol_buckets
        .iter()
        .find(|bucket| bucket.bucket == 0)
        .unwrap();
    *source.failed_member.lock().unwrap() = bucket.pages.last().copied();
    let new_text = colliding_texts().pop().unwrap();
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    let frontier = table.interner_snapshot().identity_frontier();
    let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
        new_text.as_bytes().to_vec().into_boxed_slice(),
    ));
    let failure = table.admit_field_text(&execution, &input).unwrap_err();
    let crate::symbols::data::RelationalFieldTextAdmissionDenial::StorageUnavailable(failure) =
        failure
    else {
        panic!("late bucket failure must remain typed: {failure:?}");
    };
    assert_eq!(failure.cause, Cause::Damaged);
    assert_eq!(table.interner_snapshot().identity_frontier(), frontier);
    assert!(table.with_read(|symbols| !symbols.contains(&new_text)));
    let reads = source.reads.load(Ordering::SeqCst);
    assert!(reads > 1, "the failed page follows admitted bucket pages");
    *source.failed_member.lock().unwrap() = None;
    assert!(matches!(table.admit_field_text(&execution, &input),
        Err(crate::symbols::data::RelationalFieldTextAdmissionDenial::StorageUnavailable(observed)) if observed == failure));
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
    assert_eq!(table.interner_snapshot().identity_frontier(), frontier);
}

#[test]
fn authenticated_page_with_the_wrong_id_range_is_refused_before_import() {
    let (_, mut directory, source, _) = dictionary();
    let page = &mut directory.manifest.symbols[0];
    page.first += 1;
    let symbol = Symbol(page.last as u32);
    let table = RuntimeSymbolTable::default();
    table.install_cold_pages(&directory).unwrap();
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    let failure = table.demand_symbol_ids(&execution, [symbol]).unwrap_err();
    assert!(matches!(failure, Failure::StorageUnavailable(error) if error.cause == Cause::Damaged));
    assert!(table.snapshot().entries.is_empty());
    let reads = source.reads.load(Ordering::SeqCst);
    assert_eq!(table.demand_symbol_ids(&execution, [symbol]), Err(failure));
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
}

#[test]
fn cached_page_shape_is_rechecked_for_each_directory_selection() {
    let (table, directory, source, _) = collision_dictionary();
    let locator = directory
        .manifest
        .symbol_buckets
        .iter()
        .find(|entry| entry.bucket == 0)
        .unwrap()
        .pages[0];
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    let cold = RuntimeSymbolResidency::new(
        directory.manifest.symbols.clone(),
        directory.manifest.symbol_buckets.clone(),
        source.clone(),
    )
    .unwrap();
    cold.demand_page(
        &table,
        &execution,
        locator,
        SymbolPageSelection::HashBucket(0),
    )
    .unwrap();
    let reads = source.reads.load(Ordering::SeqCst);
    let page = crate::durability::checkpoints::symbol_pages::readmit(
        &execution,
        &source.members[&locator],
        locator,
        SymbolPageSelection::HashBucket(0).subject(),
    )
    .unwrap();
    let selection = SymbolPageSelection::IdRange {
        first: page.entries.first().unwrap().0,
        last: page.entries.last().unwrap().0,
    };
    cold.demand_page(&table, &execution, locator, selection)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        reads,
        "shared page needs no second read"
    );
    let failure = cold
        .demand_page(
            &table,
            &execution,
            locator,
            SymbolPageSelection::HashBucket(1),
        )
        .unwrap_err();
    assert!(matches!(failure, Failure::StorageUnavailable(error) if error.cause == Cause::Damaged));
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
    assert_eq!(
        cold.demand_page(&table, &execution, locator, selection),
        Err(failure)
    );
}

#[test]
fn an_id_admitted_page_cannot_authorize_a_misrouted_negative_intern_probe() {
    use super::super::super::symbol_table::SymbolReservationDenial;
    let (_, mut directory, source, _) = collision_dictionary();
    let locator = directory
        .manifest
        .symbol_buckets
        .iter()
        .find(|entry| entry.bucket == 0)
        .unwrap()
        .pages[0];
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    let page = crate::durability::checkpoints::symbol_pages::readmit(
        &execution,
        &source.members[&locator],
        locator,
        SymbolPageSelection::HashBucket(0).subject(),
    )
    .unwrap();
    let first = page.entries.first().unwrap().0;
    directory.manifest.symbols = vec![RelationalCheckpointPage {
        first: first.0 as u64,
        last: page.entries.last().unwrap().0 .0 as u64,
        member: locator,
    }];
    // Keep the producer's authentic bytes and deliberately misroute their
    // bucket descriptor. Range admission alone must prove no negative lookup.
    directory.manifest.symbol_buckets = vec![RelationalCheckpointSymbolBucket {
        bucket: 1,
        pages: vec![locator],
    }];
    let table = RuntimeSymbolTable::default();
    table.install_cold_pages(&directory).unwrap();
    table.demand_symbol_ids(&execution, [first]).unwrap();
    let new_text = (0..)
        .map(|id| format!("misrouted-new-spelling-{id}"))
        .find(|text| Sha256::digest(text.as_bytes())[0] == 1)
        .unwrap();
    let before = table.interner_snapshot();
    let reads = source.reads.load(Ordering::SeqCst);
    let mut pending = table.begin_pending_meanings();
    let SymbolReservationDenial::StorageUnavailable(failure) =
        pending.reserve(&new_text).unwrap_err()
    else {
        panic!("misrouted cached bucket must refuse before issuance");
    };
    assert_eq!(failure.cause, Cause::Damaged);
    assert_eq!(
        table.interner_snapshot(),
        before,
        "a refused probe issues no identity or meaning"
    );
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        reads,
        "negative probe performs no I/O"
    );
    assert_eq!(
        table.demand_text(&execution, &new_text),
        Err(Failure::StorageUnavailable(failure))
    );
    assert!(
        matches!(pending.reserve(&new_text), Err(SymbolReservationDenial::StorageUnavailable(observed)) if observed == failure)
    );
    assert_eq!(table.interner_snapshot(), before);
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
}
