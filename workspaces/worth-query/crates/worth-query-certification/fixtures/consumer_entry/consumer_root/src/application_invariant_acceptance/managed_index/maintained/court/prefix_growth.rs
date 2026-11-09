//! Real prefix populations, whole original SDK invoices and authored membership.
use super::super::{authoring, geometry, schema::Catalog};
use super::*;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
mod population;
mod selection;
mod work_attribution;
type Principal = WorthQueryAuthenticatedExternalPrincipal<Catalog>;

pub(in super::super) fn invoice<T>(operation: impl FnOnce() -> T) -> (T, u64) {
    invoice_observing(|_| {}, operation)
}
fn invoice_observing<T>(
    on_context: impl FnOnce(&worth_execution::MapKernelContext<'_, '_>),
    operation: impl FnOnce() -> T,
) -> (T, u64) {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let (result, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&original), |context| {
            on_context(context);
            operation()
        })
        .unwrap();
    (result, report.charged_work())
}
#[test]
fn real_prefix_width_small_batches_and_sparse_queries_preserve_both_roots() {
    let small = journey(32, 0, true);
    let doubled = journey(64, 0, false);
    eprintln!("prefix small original edit invoices32={small:?},64={doubled:?}");
    local_guard(small, doubled);
}
#[test]
fn prefix_width_growth_keeps_local_edits_bounded_beside_unrelated_source_names() {
    let small = journey(1_000, 0, false);
    let doubled = journey(2_000, 0, false);
    local_guard(small, doubled);
    let beside = journey(2_000, 4_096, false);
    eprintln!("prefix original edit invoices1000={small:?},2000={doubled:?},2000+4096={beside:?}");
    for (base, remote) in [(doubled.0, beside.0), (doubled.1, beside.1)] {
        assert!(
            remote <= base * 3 / 2,
            "unrelated names must not dominate the whole original edit invoice"
        );
    }
}
fn local_guard(first: (u64, u64), doubled: (u64, u64)) {
    for (base, grown) in [(first.0, doubled.0), (first.1, doubled.1)] {
        assert!(
            base > 0 && grown <= base * 5 / 4,
            "local edit work must keep the predeclared25% doubling guard: {base}->{grown}"
        );
    }
}
fn journey(local: u32, remote: u32, invalid_batches: bool) -> (u64, u64) {
    let provider = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(provider.clone()));
    let world = host::install_growth(
        provider.clone(),
        rules.clone(),
        geometry::Configuration::default(),
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    if invalid_batches {
        population::invalid_batches(&world, &principal, &scope, &provider);
    }
    let items = population::items(local, remote);
    population::populate(&world, &items);
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    selection::census(&world, &principal, &scope, &items, "shared");
    // B stays publicly unopened. An explicit cold A rebuild is OUTSIDE every
    // warm invoice and exercises the actual full Source build above8/16 rows.
    let a = request
        .query(find("A", "sh"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let ((), cold_work) = invoice(|| {
        request.query(find("A", "sh")).evict_index(&a).unwrap();
        request.query(find("A", "sh")).rebuild_index(&a).unwrap();
    });
    eprintln!(
        "prefix cold eviction/rebuild Source={} original SDK={cold_work}",
        items.len() + 1
    );
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    assert_eq!(
        provider.canonical_rows.load(Ordering::SeqCst),
        items.len() + 1
    );
    selection::check(
        &world, &principal, &scope, &provider, &items, "shared", "before",
    );
    let counts = provider.totals();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    rules.records.lock().unwrap().clear();
    let updates = provider.updates.load(Ordering::SeqCst);
    let original = request.query(find("A", "sh")).execute().unwrap();
    let observation = original.into_result_set_observation().unwrap();
    let (denied, refused_work) = work_attribution::invoice(&provider, || {
        request
            .advanced()
            .mutate(rename("sh", "taken"))
            .expect_result_set(observation.clone())
            .idempotency(&0x919d01u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    require_violation(denied);
    assert_eq!(
        provider.updates.load(Ordering::SeqCst),
        updates + 2,
        "only x's actual two Source scopes prepare"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    {
        let records = rules.records.lock().unwrap();
        assert!(!records.is_empty() && records.len() <= 2);
        assert!(records
            .iter()
            .all(|record| record.updates_before_callback >= updates + 2));
        assert!(records
            .iter()
            .any(|record| values(&record.after) == expected("B", "taken", "taken")));
    }
    selection::check(
        &world,
        &principal,
        &scope,
        &provider,
        &items,
        "shared",
        "after refusal",
    );
    rules.records.lock().unwrap().clear();
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authenticate(&world, &fresh_scope);
    let fresh = world
        .runtime()
        .request(&fresh_principal, &fresh_scope)
        .on_branch(world.current_world());
    let (committed, committed_work) = work_attribution::invoice(&provider, || {
        fresh
            .advanced()
            .mutate(rename("sh", "free"))
            .expect_result_set(observation)
            .idempotency(&0x919d02u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    assert!(
        matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{committed:?}"
    );
    assert_eq!(provider.updates.load(Ordering::SeqCst), updates + 4);
    {
        let records = rules.records.lock().unwrap();
        assert_eq!(records.len(), 2);
        assert!(records
            .iter()
            .all(|record| record.updates_before_callback >= updates + 4));
        assert!(records
            .iter()
            .any(|record| values(&record.after) == expected("B", "free", "taken")));
    }
    selection::census(&world, &fresh_principal, &fresh_scope, &items, "free");
    selection::check(
        &world,
        &fresh_principal,
        &fresh_scope,
        &provider,
        &items,
        "free",
        "after commit",
    );
    let b = fresh
        .query(find("B", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(
        values(fresh.query(find("B", "")).execute().unwrap().rows()),
        expected("B", "free", "")
    );
    assert_eq!(
        provider.totals().0,
        counts.0 + 4,
        "each of the four actual updates requests its finite writer capacity"
    );
    assert_eq!(
        provider.totals().1,
        counts.1,
        "warm edit/read/B attach cannot hide a reconstruction"
    );
    a.close();
    b.close();
    eprintln!("prefix real Source={} local={local} remote={remote} refusal/commit SDK=({refused_work},{committed_work})", items.len()+1);
    (refused_work, committed_work)
}

fn authenticate(world: &host::Host, scope: &WorthQueryRequestScope) -> Principal {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}

mod ordered_chain;
