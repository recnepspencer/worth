//! Real directory startup/read/owning close, extended by the ordinary commit seam.
mod changed_build;
mod correction_material;
mod directory_capabilities;
mod program_support;
mod workflow_configuration;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::ApplicationBuildIdentity,
    primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};

#[test]
fn fresh_directory_home_journey() {
    let location = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let request = fixture::request(
        location.clone(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        Arc::clone(&initialized),
        Arc::new(AtomicUsize::new(0)),
    );
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .expect("the original bootstrap execution opens")
    .unwrap_or_else(|refused| panic!("fresh directory {}: {refused:?}", location.display()));
    assert!(home.is_directory());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let seeded = runtime
            .request(&principal, &scope)
            .query(fixture::balance_query::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .expect("the real authored Native field is seeded");
        assert_eq!(seeded.rows(), &[7]);
        drop(seeded);
        let outcome = runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 8,
            })
            .without_source()
            .idempotency(&0x484f4d45_u64)
            .execute_in_program(
                runtime,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the real Directory World owns the ordinary Native commit");
        let worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
            panic!("one genuine ordinary commit: {outcome:?}");
        };
        assert_eq!(result, 8);
        let observed = runtime
            .request(&principal, &scope)
            .query(fixture::balance_query::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .expect("the same actual Home selects the ordinary mutation");
        assert_eq!(observed.rows(), &[8]);
        assert_eq!(
            observed.receipt().inspect().basis().version(),
            receipt.commit_reference().version_id.as_u64()
        );
    }
    fresh_close::verify(home, &execution);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
}

pub(crate) mod pending_support;

#[test]
fn directory_pre_effect_pending_retains_key_through_handoff_and_reopen_requirement() {
    use pending_support::*;
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(execution, Arc::clone(&contacts));
    let runtime = home.runtime();
    let (cancel, stopped_scope) = scope();
    let pause = runtime
        .runtime()
        .pause_after_application_candidate_preparation_for_test(std::num::NonZeroUsize::MIN);
    let original = std::thread::scope(|threads| {
        let controller = threads.spawn(move || {
            assert!(
                pause.wait_until_reached(Duration::from_secs(20)),
                "the actual candidate reaches pre-effect preparation"
            );
            cancel.cancel();
            pause.release();
        });
        let original = pending(set(runtime, &stopped_scope, 0x901, 8));
        controller.join().unwrap();
        original
    });
    assert_eq!(original.stage(), worth_query_host::facade::primary_graph::WorthQueryProductPreparationStage::PreparingQueryCandidate);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let (_, live) = scope();
    let duplicate = pending(set(runtime, &live, 0x901, 8));
    assert_eq!(duplicate.record_handle(), original.record_handle());
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        1,
        "a live original key cannot replay the supplier"
    );
    let duplicate = match duplicate.try_into_recovery() {
        Ok(_) => panic!("the observation cannot steal the original checked-out owner"),
        Err(duplicate) => duplicate,
    };
    assert_eq!(
        balance(runtime, &live),
        7,
        "cancellation precedes Native effect"
    );
    let recovery = original
        .try_into_recovery()
        .unwrap_or_else(|pending| panic!("original consuming handoff: {pending:?}"));
    let effects = recovery
        .inspect()
        .expect("the same World catalog retains actual original custody");
    assert_reopen_actions(&effects);
    assert_eq!(effects.owner_effect_count(), 0);
    drop(effects);
    let handle = recovery.record_handle().clone();
    let recovery = reopen_required(
        runtime
            .runtime()
            .release_product_publication_recovery(recovery, 0),
    );
    assert_eq!(recovery.record_handle(), &handle);
    let retained = pending(set(runtime, &live, 0x901, 8));
    assert_eq!(retained.record_handle(), &handle);
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        1,
        "reopen requirement preserves the original key fence"
    );
    assert_eq!(balance(runtime, &live), 7);
    drop(retained);
    drop(duplicate);
    drop(recovery);
    // Recovered disposition and explicit owning close remain separate obligations.
}

#[test]
fn directory_post_native_unwind_retains_original_key_without_handler_replay() {
    use pending_support::*;
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(execution, Arc::clone(&contacts));
    let runtime = home.runtime();
    let (_, live) = scope();
    runtime
        .runtime()
        .world_operation_control_for_test()
        .panic_before_product_compare_once_for_test();
    let original = pending(set(runtime, &live, 0x902, 9));
    assert_eq!(
        original.stage(),
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationStage::Publishing
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let duplicate = pending(set(runtime, &live, 0x902, 9));
    assert_eq!(duplicate.record_handle(), original.record_handle());
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        1,
        "actual post-effect custody prevents handler replay"
    );
    assert_eq!(
        balance(runtime, &live),
        7,
        "unpublished Native effects are not a World-published application result"
    );
    drop(duplicate);
    let recovery = original
        .try_into_recovery()
        .unwrap_or_else(|pending| panic!("original unwind handoff: {pending:?}"));
    let effects = recovery
        .inspect()
        .expect("the actual callee unwind survived in the original catalog");
    assert_reopen_actions(&effects);
    assert!(
        effects.owner_effect_count() > 0,
        "the control interrupts after actual Native effects"
    );
    drop(effects);
    let handle = recovery.record_handle().clone();
    let recovery = reopen_required(
        runtime
            .runtime()
            .release_product_publication_recovery(recovery, 0),
    );
    assert_eq!(recovery.record_handle(), &handle);
    let retained = pending(set(runtime, &live, 0x902, 9));
    assert_eq!(retained.record_handle(), &handle);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(runtime, &live), 7);
    drop(retained);
    drop(recovery);
    // Only genuine reopened custody can admit a recovered disposition.
}

mod dispatch_custody;
pub(crate) mod reopening;

mod recovered_custody;

mod close_admission;
mod handler_retirement;

mod fresh_source;

mod fresh_close;
mod program_transition;
mod transition_required;

mod signing_owner_retirement;

#[test]
fn failed_managed_cleanup_records_live_release_and_discharges_after_directory_reopen() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted, WorthQueryHomeRetirementDenial,
    };
    let location = fixture::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let initialized = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        location.clone(),
        build,
        initialized.clone(),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("cleanup Home: {cause:?}"));
    let schedule = home
        .runtime()
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let (_, scope) = pending_support::scope();
    let required = match pending_support::set(home.runtime(), &scope, 0x4847_0701, 29).unwrap() {
        Outcome::Commit(Uncommitted::CleanupRequired(required)) => required,
        other => panic!("the actual native cleanup must refuse: {other:?}"),
    };
    drop(schedule);
    assert!(required.cleanup().durability_denial().is_none());
    let identity = required
        .cleanup()
        .obligation_identity()
        .expect("acknowledged cleanup identity");
    let failure = required.cleanup().failure().kind();
    let run = required.cleanup().failure().run_identity().to_owned();
    let rows = home.runtime().runtime().managed_cleanup_obligations();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].identity, identity);
    assert_eq!(rows[0].failure, failure);
    assert_eq!(rows[0].run, run);
    assert!(rows[0].release.is_none());
    let original = rows[0]
        .original
        .expect("actual World original remains associated with failed cleanup");
    let refused = home
        .close(&execution)
        .err()
        .expect("unreleased live resources refuse close");
    assert!(matches!(
        refused.query_denial(),
        Some(WorthQueryHomeRetirementDenial::ManagedCleanup(_))
    ));
    let home = refused
        .resume()
        .unwrap_or_else(|cause| panic!("same original cleanup Home: {cause:?}"));
    let (business, cleanup) = required.into_parts();
    let receipt = home
        .runtime()
        .runtime()
        .with_host_advancement(|phase| cleanup.retry_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|cause| panic!("actual owner-thread release: {cause:?}"));
    assert!(receipt.inspection().resources_released());
    assert_eq!(receipt.inspection().run_identity(), run);
    assert!(home.runtime().runtime().managed_cleanup_obligations()[0]
        .release
        .is_some());
    drop(receipt);
    drop(business);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("recorded resource release close: {cause:?}"))
        .reclaim();
    let request =
        fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
            panic!("initializer replayed")
        });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("fresh cleanup owner: {cause:?}"));
    let rows = home.runtime().runtime().managed_cleanup_obligations();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].identity, identity);
    assert_eq!(rows[0].original, Some(original));
    assert_eq!(rows[0].failure, failure);
    assert_eq!(rows[0].run, run);
    assert!(
        rows[0].release.is_some(),
        "only the recorded actual release admits discharge"
    );
    home.runtime()
        .runtime()
        .with_host_advancement(|phase| {
            home.runtime()
                .runtime()
                .discharge_managed_cleanup(&phase, identity)
        })
        .unwrap()
        .unwrap();
    assert!(home
        .runtime()
        .runtime()
        .managed_cleanup_obligations()
        .is_empty());
    let (_, scope) = pending_support::scope();
    assert!(matches!(
        pending_support::set(home.runtime(), &scope, 0x4847_0701, 29).unwrap(),
        Outcome::RecoveredCommitted(_)
    ));
    assert_eq!(pending_support::balance(home.runtime(), &scope), 29);
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        1,
        "cleanup never reexecutes the business effect"
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("discharged cleanup close: {cause:?}"))
        .reclaim();
}
