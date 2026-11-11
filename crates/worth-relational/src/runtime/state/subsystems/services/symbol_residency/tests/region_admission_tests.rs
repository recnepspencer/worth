//! A region's dictionary closure is admitted once per runtime and view.
use super::*;

#[test]
fn region_symbol_admission_is_retryable_and_owned_by_each_fork() {
    let (table, _, source, all) = dictionary();
    let fork = table.detached_owner_snapshot();
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&execution, &runtime, "region-admission");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let lease = root
        .demand_region_head(&execution, entity.partition_id)
        .unwrap()
        .unwrap();
    let symbol = all.entries[0].0;
    *source.failure.lock().unwrap() = Some(Cause::MemoryExhausted);
    assert!(
        matches!(table.demand_region_symbols(&execution, lease.symbol_admission_identity(),
        || BTreeSet::from([symbol])), Err(Failure::StorageUnavailable(failure))
        if failure.cause == Cause::MemoryExhausted)
    );
    *source.failure.lock().unwrap() = None;
    table
        .demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
            BTreeSet::from([symbol])
        })
        .unwrap();
    table
        .demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
            panic!("a warm region must not rescan its dictionary closure")
        })
        .unwrap();
    let admitted_fork = table.detached_owner_snapshot();
    admitted_fork
        .demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
            panic!("an admitted fork carries both meanings and the accepted closure")
        })
        .unwrap();
    assert!(fork.with_read(|symbols| symbols.retained_text(symbol).is_none()));
    let required = std::cell::Cell::new(false);
    fork.demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
        required.set(true);
        BTreeSet::from([symbol])
    })
    .unwrap();
    assert!(
        required.get(),
        "another runtime must admit this view independently"
    );
    assert!(fork.with_read(|symbols| symbols.retained_text(symbol).is_some()));
}

#[test]
fn dictionary_snapshot_shares_directory_and_page_owners_without_sharing_later_edits() {
    let (table, directory, _, all) = dictionary();
    let original = crate::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&original);
    let parent_pages =
        RuntimeSymbolResidency::from_selected_directory(&directory, &request).unwrap();
    parent_pages
        .demand_ids(
            &table,
            &request,
            [
                all.entries.first().unwrap().0,
                all.entries.last().unwrap().0,
            ],
        )
        .unwrap();
    let fork = table.detached_owner_snapshot();
    let (fork_pages, _) = parent_pages.detached_with_owner(|| table.interner_snapshot());
    assert!(parent_pages.by_id.shares_backing(&fork_pages.by_id));
    assert!(parent_pages.by_value.shares_backing(&fork_pages.by_value));
    {
        let parent = parent_pages.pages.lock().unwrap();
        let detached = fork_pages.pages.lock().unwrap();
        assert_eq!(parent.len(), detached.len());
        for (member, accepted) in parent.iter() {
            assert!(
                std::ptr::eq(accepted, detached.get(member).unwrap()),
                "a detached page result must retain the same persistent value owner"
            );
        }
    }
    table.with_write(|symbols| symbols.intern("after-read-snapshot").unwrap());
    assert!(fork.with_read(|symbols| symbols.symbol("after-read-snapshot").is_none()));
    for symbol in [
        all.entries.first().unwrap().0,
        all.entries.last().unwrap().0,
    ] {
        assert_eq!(
            fork.with_read(|symbols| symbols.resolve(symbol).map(str::to_owned)),
            table.with_read(|symbols| symbols.resolve(symbol).map(str::to_owned))
        );
    }
}

#[test]
fn candidate_admission_transfers_only_after_publication_and_never_borrows_later_owner_marks() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    let entity = create_entity(&execution, &runtime, "transfer-owner");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let lease = root
        .demand_region_head(&execution, entity.partition_id)
        .unwrap()
        .unwrap();
    let symbol = runtime
        .services
        .symbols
        .with_read(|symbols| symbols.symbol("name"))
        .unwrap();
    let (table, _, _, _) = checkpoint_dictionary(&execution, &runtime);
    let ticket = crate::storage::residency::SymbolPublicationAdmission::prepare(&root).unwrap();
    let before_acceptance = table.detached_owner_snapshot();
    let pending = table.begin_pending_meanings();
    let binding = pending.binding();
    binding
        .demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
            BTreeSet::from([symbol])
        })
        .unwrap();
    assert!(!table.region_symbols_admitted(lease.symbol_admission_identity()));
    assert!(table.with_read(|symbols| symbols.retained_text(symbol).is_none()));
    // The original pending base must not acquire a later binding's marks when
    // binding() replaces its interner with that base's frozen meaning snapshot.
    let second_binding = pending.binding();
    assert!(!second_binding.region_symbols_admitted(lease.symbol_admission_identity()));
    assert!(second_binding.with_read(|symbols| symbols.retained_text(symbol).is_none()));
    binding
        .demand_symbol_ids(&execution, ticket.required())
        .unwrap();
    binding.register_symbol_publication(ticket.key());
    assert!(!table.symbol_publication_admitted(ticket.key()));
    assert!(!second_binding.symbol_publication_admitted(ticket.key()));
    let sealed = pending
        .seal_for_publication(
            &crate::symbols::data::SymbolTableSnapshot { entries: vec![] },
            &binding,
            [],
        )
        .unwrap();
    assert!(!table.region_symbols_admitted(lease.symbol_admission_identity()));
    sealed.publish().unwrap();
    assert!(table.symbol_publication_admitted(ticket.key()));
    assert!(!before_acceptance.symbol_publication_admitted(ticket.key()));
    assert!(!before_acceptance
        .admit_region_from_publication(&execution, lease.symbol_admission_identity(), ticket.key())
        .unwrap());
    assert!(table
        .admit_region_from_publication(&execution, lease.symbol_admission_identity(), ticket.key())
        .unwrap());
    assert!(ticket
        .for_region(root.commit_change().unwrap(), entity.partition_id)
        .is_some());
    let other = runtime_with_test_schema();
    create_entity(&execution, &other, "another-publication");
    let other_root = other
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    assert!(ticket
        .for_region(other_root.commit_change().unwrap(), entity.partition_id)
        .is_none());
    assert!(table
        .detached_owner_snapshot()
        .symbol_publication_admitted(ticket.key()));
    assert!(table.region_symbols_admitted(lease.symbol_admission_identity()));
    assert_eq!(
        table.with_read(|symbols| symbols.resolve(symbol).map(str::to_owned)),
        Some("name".to_owned())
    );
    let after = table.begin_pending_meanings().binding();
    after
        .demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
            panic!("published admission is inherited with its meaning")
        })
        .unwrap();
}

#[test]
fn publication_refuses_marks_whose_frozen_dictionary_meaning_was_erased() {
    let original = crate::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&original);
    let (table, _, _, all) = dictionary();
    table
        .demand_symbol_ids(&request, [all.entries[0].0])
        .unwrap();
    let pending = table.begin_pending_meanings();
    let altered = pending.binding();
    altered.with_write(|symbols| *symbols = Default::default());
    assert!(pending
        .seal_for_publication(
            &crate::symbols::data::SymbolTableSnapshot { entries: vec![] },
            &altered,
            [],
        )
        .is_err());
}

#[test]
fn owned_successor_without_predecessor_admission_refuses_instead_of_rebuilding() {
    let original = crate::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&original);
    let runtime = durable_runtime_with_test_schema();
    let entity = create_entity(&request, &runtime, "missing-predecessor");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let previous = root
        .demand_region_full(&request, entity.partition_id)
        .unwrap()
        .unwrap();
    let (table, _, source, namespace) = checkpoint_dictionary(&request, &runtime);
    update_entity_and_release_snapshot(&request, &runtime, entity, "changed requirements");
    let next_root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let next = next_root
        .demand_region_full(&request, entity.partition_id)
        .unwrap()
        .unwrap();
    let before_reads = source.reads.load(Ordering::SeqCst);
    assert!(
        matches!(next.admit_symbols(&request, &table), Err(Failure::StorageUnavailable(failure)) if failure.cause == Cause::MissingSymbolMaintenancePredecessor)
    );
    assert_eq!(source.reads.load(Ordering::SeqCst), before_reads);
    // A separate cold dictionary has no accepted lease identity yet. Explicit
    // readmission of the producer's complete checkpoint namespace establishes
    // the predecessor; the ordinary successor never reconstructs that fact.
    table
        .demand_region_symbols(&request, previous.symbol_admission_identity(), || {
            namespace.entries.iter().map(|(id, _)| *id).collect()
        })
        .unwrap();
    next.admit_symbols(&request, &table).unwrap();
}

#[test]
fn a_refused_page_cache_update_preserves_the_previous_shared_proof_and_dictionary() {
    let (table, directory, source, all) = dictionary();
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let pages = RuntimeSymbolResidency::from_selected_directory(&directory, &execution).unwrap();
    let symbol = all.entries.first().unwrap().0;
    pages.demand_ids(&table, &execution, [symbol]).unwrap();
    let page = RelationalCheckpointPage::find(&pages.by_id, u64::from(symbol.0)).unwrap();
    let accepted = pages.pages.lock().unwrap().data.clone();
    let dictionary = table.interner_snapshot();
    let reads = source.reads.load(Ordering::SeqCst);
    let budget = worth_execution::SerialMemoryBudget::new(Some(0));
    let refused_host = worth_execution::SerialRequest::from_memory(
        budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let refused = worth_execution::ExecutionRequest::serial(&refused_host);
    let result = pages.demand_page(
        &table,
        &refused,
        page.member,
        SymbolPageSelection::IdRange {
            first: Symbol(page.first as u32 + 1),
            last: Symbol(page.last as u32),
            id_digest: [0; 32],
        },
    );
    assert!(matches!(result, Err(Failure::Request(_))));
    let current = pages.pages.lock().unwrap();
    assert!(std::ptr::eq(
        accepted.get(&page.member).unwrap(),
        current.get(&page.member).unwrap(),
    ));
    assert!(matches!(current.get(&page.member), Some(Ok(_))));
    drop(current);
    assert_eq!(table.interner_snapshot(), dictionary);
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
