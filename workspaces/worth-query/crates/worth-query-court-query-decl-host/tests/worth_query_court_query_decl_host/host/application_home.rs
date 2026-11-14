//! Real directory startup/read/owning close, extended by the ordinary commit seam.
mod changed_build;
mod correction_material;
mod directory_capabilities;
mod managed_cleanup;
mod managed_runs;
mod program_support;
mod temporal;
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
    {
        use worth_foundational::facade::DurableStateFamily as Family;
        use worth_query_host::facade::{
            primary_graph::WorthQueryOperationDurableCapability as Capability,
            product::WorthQueryDurableStateAdmissionDenial as Denial,
        };
        let product = home
            .runtime()
            .runtime()
            .runtime_for_test()
            .product_runtime_for_test();
        for (capability, family, phase) in [
            (
                Capability::RequiredOutputDemand,
                Family::OutputDemandRecords,
                6,
            ),
            (
                Capability::ProgramAdoption,
                Family::BranchAdoptionRecovery,
                6,
            ),
            (Capability::WorkflowExecution, Family::WorkflowInstances, 7),
            (
                Capability::RetainedOriginalOperationOwner,
                Family::OriginalOperationDescription,
                9,
            ),
        ] {
            assert_eq!(
                product.admit_directory_capability_for_test(capability),
                Err(Denial::Unsupported {
                    family,
                    supported_from_phase: phase
                })
            );
        }
    }
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
