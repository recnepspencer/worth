//! One isolated real public composition: unopened scope, refusal, publication and retry.
use super::super::super::authentication;
use super::super::lifecycle;
use super::{host, mutation, prefix, query, rule};
use stats_alloc::Region;
use std::sync::{atomic::Ordering, Arc, Mutex};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_contribution::*, application_entry::*,
    primary_graph::*, runtime::ExecutionAllocationPolicy,
};

mod cancellation;
mod decision_scope;
mod membership;
mod prefix_growth;
mod retirement;
mod source_structure;
mod summary_support;
mod text_read;
mod watch;

fn find(scope: &str, prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: scope.into(),
        prefix: prefix.into(),
    }
}
fn rename(prefix: &str, value: &str) -> mutation::Rename {
    mutation::Rename {
        directory: "A".into(),
        item: "x".into(),
        prefix: prefix.into(),
        replacement: value.into(),
    }
}
fn values(rows: &[query::ItemRow]) -> Vec<(String, String)> {
    let mut values: Vec<_> = rows
        .iter()
        .map(|row| (row.identity.clone(), row.name.clone()))
        .collect();
    values.sort();
    values
}
/// Authored records are the independent exhaustive universe, not provider nodes,
/// result order, a Source descriptor, or an expected counter fitted to a run.
fn expected(scope: &str, x: &str, prefix: &str) -> Vec<(String, String)> {
    [("x", x), ("y", "taken")]
        .into_iter()
        .filter(|(id, name)| (scope == "B" || *id == "x") && name.starts_with(prefix))
        .map(|(id, name)| (id.to_owned(), name.to_owned()))
        .collect()
}
#[test]
fn public_maintained_prefix_prepares_unopened_scope_before_rule_and_publishes_matching_roots() {
    lifecycle::isolated_allocation_case(run, concat!(module_path!(), "::public_maintained_prefix_prepares_unopened_scope_before_rule_and_publishes_matching_roots"));
}
fn run() {
    // Prime process-local schema/codec caches outside the allocation court.
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let finished = region.change();
    assert_eq!(finished.bytes_allocated, finished.bytes_deallocated,
        "all relevant requests, snapshots, public handles, Native roots, runtime caches and observers closed");
}
fn journey() {
    let provider = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
    let world = host::install(Arc::clone(&provider), Arc::clone(&rules));
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
    // Only A is opened. B's root and invocation cannot come from public sessions.
    let a = request
        .query(find("A", "sh"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(a.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let original = request.query(find("A", "sh")).execute().unwrap();
    assert_eq!(values(original.rows()), expected("A", "shared", "sh"));
    let warm = request
        .advanced()
        .mutate(rename("sh", "shared"))
        .expect_result_set(original.result_set_observation().unwrap().clone())
        .idempotency(&0x919200u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(warm, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "{warm:?}"
    );
    drop(warm);
    rules.records.lock().unwrap().clear();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let reached = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&reached);
    let refusal_source = request.query(find("A", "sh")).execute().unwrap();
    let updates = provider.updates.load(Ordering::SeqCst);
    let memory = accounted_request();
    let observed_memory = Arc::clone(&memory);
    let baseline = memory.charged_memory_bytes();
    let bases = world.runtime().application_query_basis_observer();
    let active = bases.observe().active();
    *provider.after_first_node.lock().unwrap() = Some(Box::new(move || {
        *observed.lock().unwrap() = Some(observed_memory.charged_memory_bytes());
    }));
    let denied = request
        .advanced()
        .mutate(rename("sh", "taken"))
        .expect_result_set(refusal_source.result_set_observation().unwrap().clone())
        .idempotency(&0x919201u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    require_violation(denied);
    assert!(
        reached.lock().unwrap().is_some(),
        "a genuine node write preceded the domain refusal"
    );
    let denied_b = {
        let records = rules.records.lock().unwrap();
        assert!(
            (1..=2).contains(&records.len()),
            "evaluation stops at the actual domain refusal"
        );
        assert!(
            records
                .iter()
                .all(|record| record.updates_before_callback >= updates + 2),
            "both roots were staged before the FIRST rule callback"
        );
        let b = records
            .iter()
            .find(|record| values(&record.after) == expected("B", "taken", "taken"))
            .expect("the real B proposal contains both x and y");
        assert_eq!(
            values(b.before.as_ref().expect("B has an actual committed root")),
            expected("B", "shared", "taken")
        );
        if let Some(a) = records
            .iter()
            .find(|record| values(&record.after) == expected("A", "taken", "taken"))
        {
            assert_ne!(a.scope, b.scope);
            assert!(a.before.as_ref().unwrap().is_empty());
        }
        b.scope
    };
    rules.records.lock().unwrap().clear();
    assert!(
        *reached.lock().unwrap().as_ref().unwrap() > baseline,
        "the original request held actual staged backing at the node write"
    );
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "domain refusal releases original-request staging reservations"
    );
    assert_eq!(
        bases.observe().active(),
        active,
        "the failed decision query releases its admitted basis"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(
        values(request.query(find("A", "")).execute().unwrap().rows()),
        expected("A", "shared", "")
    );

    let source = request
        .query(find("A", "sh"))
        .execute()
        .unwrap()
        .into_result_set_observation()
        .expect("the admitted managed query retains its complete result-set observation");
    let committed = request
        .advanced()
        .mutate(rename("sh", "free"))
        .expect_result_set(source.clone())
        .idempotency(&0x919202u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{committed:?}"
    );
    drop(committed);
    {
        let records = rules.records.lock().unwrap();
        assert_eq!(
            records.len(),
            2,
            "the valid twin evaluates both prepared scopes"
        );
        let b = records
            .iter()
            .find(|record| record.scope == denied_b)
            .unwrap();
        assert_eq!(
            values(b.before.as_ref().unwrap()),
            expected("B", "shared", "taken")
        );
        assert_eq!(values(&b.after), expected("B", "free", "taken"));
        let a = records
            .iter()
            .find(|record| record.scope != denied_b)
            .unwrap();
        assert!(a.before.as_ref().unwrap().is_empty());
        assert!(a.after.is_empty());
    }
    rules.records.lock().unwrap().clear();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let stale = request
        .advanced()
        .mutate(rename("sh", "wrong"))
        .expect_result_set(source)
        .idempotency(&0x919205u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = stale.expect_err("changed complete Source refuses before handler effects")
    else {
        panic!("the retained Source must refuse for actual currentness")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert!(
        rules.records.lock().unwrap().is_empty(),
        "stale source cannot reach rule callbacks"
    );
    let build_counts = (
        provider.capacity.load(Ordering::SeqCst),
        provider.builds.load(Ordering::SeqCst),
    );
    let b = request
        .query(find("B", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(b.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let actual = request.query(find("B", "")).execute().unwrap();
    assert_eq!(values(actual.rows()), expected("B", "free", ""));
    let scan = request
        .query(query::ReadSource {
            directory: "B".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(values(scan.rows()), expected("B", "free", ""));
    assert_eq!(values(actual.rows()), values(scan.rows()));
    assert_eq!(
        (
            provider.capacity.load(Ordering::SeqCst),
            provider.builds.load(Ordering::SeqCst)
        ),
        build_counts,
        "admitted B open/read attaches the published root without Source/provider reconstruction"
    );
    drop(actual);
    drop(scan);
    cancellation::run(&world, &principal, &provider, &rules);
    a.close();
    b.close();
    drop(original);
    drop(request);
    drop(principal);
    drop(scope);
    drop(world);
    rules.records.lock().unwrap().clear();
}
fn require_violation<D: std::fmt::Debug, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
) {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("the actual B domain rule must refuse: {outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let Some(WorthQueryCustomInvariantDenial::Violation { identity, .. }) =
        denial.custom_invariant_denial()
    else {
        panic!("setup, Source-unavailable and READ failures cannot counterfeit the domain violation: {denial:?}")
    };
    assert_eq!(identity.rule_id.as_str(), "AtMostOneTakenName");
}

pub(super) fn accounted_request() -> Arc<worth_execution::ExecutionResourceLease<'static>> {
    use std::num::NonZeroUsize;
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    Arc::new(
        worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, Some(64 * 1024 * 1024), None),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        })
        .unwrap(),
    )
}
