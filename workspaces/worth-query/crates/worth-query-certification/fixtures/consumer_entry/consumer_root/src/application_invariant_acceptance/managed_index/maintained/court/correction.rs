//! Genuine recorded correction exercises the same owning Program and canonical witnesses.
use super::super::{
    correction::{self, mover, note, recorded, state},
    schema::Catalog,
};
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
mod delivery;
use delivery::*;

#[test]
fn managed_correction_requires_genuine_cancellation_for_populated_and_empty_inputs() {
    for (prefix, empty, moved) in [("sh", false, "renamed"), ("needle", true, "needle")] {
        let calls = Arc::new(prefix::Calls::default());
        let rules = Arc::new(rule::Calls::new(calls.clone()));
        let world = host::install(calls, rules);
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let source = note_source(&world, &request, prefix, empty, 0x9194_1001);
        state(&request, "recorded note", "shared");
        let changed = move_source(&world, &request, moved, 0x9194_1002);
        state(&request, "recorded note", moved);
        reverse_mover(&world, &request, &changed, "2442391555");
        state(&request, "recorded note", "shared");
        reverse_note(&world, &request, &source, "2442391556");
        state(&request, "", "shared");
    }
}

#[test]
fn managed_correction_refuses_raw_restoration_and_an_uncancelled_transient_entrant() {
    for (prefix, empty, moved) in [("sh", false, "renamed"), ("needle", true, "needle")] {
        let calls = Arc::new(prefix::Calls::default());
        let rules = Arc::new(rule::Calls::new(calls.clone()));
        let world = host::install(calls, rules);
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let source = note_source(&world, &request, prefix, empty, 0x9194_1101);
        move_source(&world, &request, moved, 0x9194_1102);
        move_source(&world, &request, "shared", 0x9194_1103);
        state(&request, "recorded note", "shared");
        refuse_note(&request, &source, "2442391812");
        state(&request, "recorded note", "shared");
    }
}

#[test]
fn managed_correction_excludes_nonmatching_authored_noops_from_empty_predicate_history() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = note_source(&world, &request, "needle", true, 0x9194_1201);
    move_source(&world, &request, "shared", 0x9194_1202);
    state(&request, "recorded note", "shared");
    reverse_note(&world, &request, &source, "2442392069");
    state(&request, "", "shared");
}

#[test]
fn managed_correction_excludes_uncancelled_irrelevant_edits_using_actual_historical_rows() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = note_source(&world, &request, "needle", true, 0x9194_1301);
    // Both real historical endpoints fail Needle. There is no performed pair.
    move_source(&world, &request, "elsewhere", 0x9194_1302);
    reverse_note(&world, &request, &source, "2442392325");
    state(&request, "", "elsewhere");
}

#[test]
fn ordinary_recorded_correction_preserves_an_intervening_disjoint_note() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let moved = move_source(&world, &request, "renamed", 0x9194_1401);
    note_source(&world, &request, "needle", true, 0x9194_1402);
    state(&request, "recorded note", "renamed");
    reverse_mover(&world, &request, &moved, "2442392580");
    state(&request, "recorded note", "shared");
}

#[test]
fn managed_correction_keeps_matching_noops_rejected_by_exact_refinement() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    // Prefix sha sees Shared, but actual exact refinement rejects it. The
    // handler's empty result therefore cannot stand in for covered influence.
    let source = note_input(
        &world,
        &request,
        note::Input {
            parent: note::Parent::Exact,
            directory: "A".into(),
            prefix: "sha".into(),
            expect_empty: true,
            replacement: "recorded note".into(),
        },
        0x9194_1501,
    );
    let noop = move_source(&world, &request, "shared", 0x9194_1502);
    state(&request, "recorded note", "shared");
    refuse_note(&request, &source, "2442392835");
    reverse_mover(&world, &request, &noop, "2442392836");
    reverse_note(&world, &request, &source, "2442392837");
    state(&request, "", "shared");
}

#[test]
fn managed_history_exclusion_preserves_independent_ordinary_authored_noop() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = note_source(&world, &request, "needle", true, 0x9194_1601);
    // Same DirectoryNote value and no Item Source impact. Ordinary field history
    // must still select this genuine authored write; present content is equal.
    let noop = note_source(&world, &request, "needle", true, 0x9194_1602);
    state(&request, "recorded note", "shared");
    refuse_note(&request, &source, "2442393091");
    reverse_note(&world, &request, &noop, "2442393092");
    reverse_note(&world, &request, &source, "2442393093");
    state(&request, "", "shared");
}

#[test]
fn managed_historical_matching_preserves_original_cancellation_and_allows_fresh_retry() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls.clone(), rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = note_source(&world, &request, "needle", true, 0x9194_1701);
    move_source(&world, &request, "elsewhere", 0x9194_1702);
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let observed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let in_match = observed.clone();
    calls.matched_sources.store(1, Ordering::SeqCst);
    *calls.after_second_match.lock().unwrap() = Some(Box::new(move || {
        in_match.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    let stopped = world
        .runtime()
        .request(&principal, &stopped_scope)
        .on_branch(world.current_world());
    let selected = stopped
        .corrections()
        .select(
            &source,
            WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN),
        )
        .unwrap();
    let denial = stopped
        .corrections()
        .reverse(&selected)
        .idempotency("2442393347")
        .limits(WorthQueryCorrectionHistoryLimits::new(
            None,
            std::num::NonZeroUsize::MIN,
        ))
        .prepare()
        .err()
        .expect("matching cancellation refuses");
    assert!(
        observed.load(Ordering::SeqCst),
        "cancellation happened in actual historical provider matching"
    );
    assert!(
        matches!(
            denial,
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::RestoredSupport(
                    WorthQueryCorrectionRestoredSupportDenial::ManagedProvider(
                        WorthQueryManagedIndexWorkDenial::Controlled(
                            WorthQueryManagedIndexStop::Request(
                                worth_execution::WorkCeilingDenial::Stopped(
                                    worth_execution::MapKernelStop::Cancelled
                                )
                            )
                        )
                    )
                )
            )
        ),
        "{denial:?}"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    state(&request, "recorded note", "elsewhere");
    reverse_note(&world, &request, &source, "2442393348");
    state(&request, "", "elsewhere");
}
