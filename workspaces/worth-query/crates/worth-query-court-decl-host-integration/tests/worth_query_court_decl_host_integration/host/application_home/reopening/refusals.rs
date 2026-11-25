//! Early partial artifacts must retain their actual owning refusal.
use super::*;
use worth_query_host::facade::{
    runtime::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial},
    ApplicationHomeQueryRootDenial, WorthQueryApplicationHomeOpenRefusedPhase,
};

#[test]
fn initializer_refusal_reopens_at_missing_world_origin_without_replaying_initializer() {
    let location = fixture::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    assert_initializer_panic_refuses_and_closes(
        location.clone(),
        build,
        contacts.clone(),
        callbacks.clone(),
        execution,
    );
    assert_missing_world_origin_holds_lease_against_competing(
        location, build, contacts, callbacks, execution,
    );
}

#[inline(never)]
fn assert_initializer_panic_refuses_and_closes(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
    callbacks: Arc<AtomicUsize>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let request = fixture::request_with_initializer(location, build, contacts, move |_, _, _| {
        callbacks.fetch_add(1, Ordering::SeqCst);
        panic!("court initializer stopped before any seed publication")
    });
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("the actual initializer refuses");
    assert!(matches!(
        refused.phase(),
        WorthQueryApplicationHomeOpenRefusedPhase::InitialState {
            cause: worth_query_execution::facade::application_home_integration::QueryHomeInitialStateDenial::Panicked,
            ..
        }
    ));
    assert!(refused.to_string().contains("Panicked"));
    refused
        .close_with_policy(worth_foundational::facade::ExecutionRequestPolicy::new(
            worth_foundational::facade::ExecutionPosture::Serial,
            worth_foundational::facade::DeterminismContract::CanonicalBitwise,
            worth_foundational::facade::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(128 * 1024 * 1024),
                None,
            ),
        ))
        .unwrap_or_else(|refused| panic!("original owning close: {refused:?}"));
}

#[inline(never)]
fn assert_missing_world_origin_holds_lease_against_competing(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
    callbacks: Arc<AtomicUsize>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let request =
        fixture::request_with_initializer(location.clone(), build, contacts.clone(), |_, _, _| {
            panic!("partial existing artifacts cannot replay initialization")
        });
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("the closed partial Home still has no World origin");
    match refused.phase() {
        WorthQueryApplicationHomeOpenRefusedPhase::ExistingWorldRoot {
            cause:
                ApplicationHomeQueryRootDenial::ExistingRoot(RuntimeWorldServiceDenial::Denied(
                    RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
                )),
            ..
        } => {}
        other => {
            panic!("the missing original World root must refuse without initialization: {other:?}")
        }
    }
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_competing_request_contended_and_close(location, build, contacts, execution);
    refused
        .close(&execution)
        .unwrap_or_else(|refused| panic!("existing owning close: {refused:?}"));
}

#[inline(never)]
fn assert_competing_request_contended_and_close(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) {
    let competing = fixture::request_with_initializer(location, build, contacts, |_, _, _| {
        panic!("held lease cannot initialize")
    });
    let competing = with_bootstrap_advancement_from_request(execution, |phase| {
        competing.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("the original refusal still holds its mutation lease");
    assert!(matches!(
        competing.phase(),
        WorthQueryApplicationHomeOpenRefusedPhase::Physical { .. }
    ));
    assert!(competing.detail().contains("MutationOwnerContended"));
    competing
        .close(&execution)
        .unwrap_or_else(|refused| panic!("competing request disposal: {refused:?}"));
}
