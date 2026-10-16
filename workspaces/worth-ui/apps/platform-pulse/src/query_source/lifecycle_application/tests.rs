use super::{PlatformPulseQueryLifecycle, PlatformPulseQueryLifecycleDenial};
use worth_ui::facade::query_binding::{
    UiProjectionObservation, WorthUiScalarProjectionSourceRecord, WorthUiStatusSourceOwner,
};

#[test]
fn committed_query_update_cannot_report_clean_close_before_presentation() {
    let owner = WorthUiStatusSourceOwner::install().expect("install authored Query owner");
    let (_, initial) = owner
        .initial_projection()
        .expect("issue initial Query fact");
    let mut lifecycle = PlatformPulseQueryLifecycle::new(owner, initial);
    let UiProjectionObservation::ApplicationScalar(initial) = lifecycle
        .issue_initial()
        .expect("issue the initial observation")
    else {
        panic!("the authored Query owner must issue an application scalar fact");
    };
    lifecycle
        .admit_publication(initial)
        .expect("admit the exact initial fact");
    let pending = lifecycle
        .advance(
            WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("first status revision"),
        )
        .expect("Query commits and issues the next fact");
    assert!(matches!(
        pending,
        UiProjectionObservation::ApplicationScalar(_)
    ));
    assert!(matches!(
        lifecycle.close(),
        Err((_, PlatformPulseQueryLifecycleDenial::UnsettledPublicationAtClose))
    ));
}

#[test]
fn cancelled_close_returns_the_whole_original_lifecycle_then_fresh_retry_closes() {
    use super::{PlatformPulseQueryOwnerState, PlatformPulseQueryMutation, PlatformPulseQueryMutationPurpose};
    use worth_query_host::facade::admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope, WorthQueryAuthenticationDenialKind};
    use worth_ui::facade::query_binding::{WorthUiStatusMutationStop, WorthUiStatusOwnerError};
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let (_, initial) = owner.initial_projection().unwrap();
    let mut lifecycle = PlatformPulseQueryLifecycle::new(owner, initial);
    let UiProjectionObservation::ApplicationScalar(initial) = lifecycle.issue_initial().unwrap() else { panic!("real scalar fact expected"); };
    lifecycle.admit_publication(initial).unwrap();
    let before = lifecycle.owner.read_status().unwrap();
    let operation = lifecycle.owner.prepare_source(WorthUiScalarProjectionSourceRecord::new("UNEXECUTED", 1).unwrap(), &super::fresh_query_scope()).unwrap();
    lifecycle.state = PlatformPulseQueryOwnerState::Mutation(PlatformPulseQueryMutation::new(operation, PlatformPulseQueryMutationPurpose::Source));
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    let (lifecycle, denial) = match lifecycle.close_with_scope(&stopped) {
        Err(refused) => refused,
        Ok(_) => panic!("cancelled authentication cannot close actual operation custody"),
    };
    let lifecycle = *lifecycle;
    assert!(matches!(denial, PlatformPulseQueryLifecycleDenial::MutationPending));
    assert!(matches!(lifecycle.mutation_stop(), Some(WorthUiStatusMutationStop::Setup(WorthUiStatusOwnerError::Authentication(denial))) if denial.kind() == WorthQueryAuthenticationDenialKind::Cancelled));
    let current = lifecycle.owner.read_status().unwrap();
    assert_eq!(current.value(), before.value(), "close must not execute the original command");
    assert_eq!(current.query_receipt().inspect().basis().version(), before.query_receipt().inspect().basis().version());
    match &lifecycle.state {
        PlatformPulseQueryOwnerState::Mutation(mutation) => assert!(matches!(mutation.purpose(), PlatformPulseQueryMutationPurpose::Source)),
        _ => panic!("same native operation and purpose must stay owned"),
    }
    let closed = match lifecycle.close_with_scope(&super::fresh_query_scope()) {
        Ok(closed) => closed,
        Err(_) => panic!("fresh authority closes the actual unexecuted operation and original source owner"),
    };
    assert!(closed.owner_terminal());
}

#[test]
fn indeterminate_close_keeps_the_original_query_owner() {
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let (_, initial) = owner.initial_projection().unwrap();
    let mut lifecycle = PlatformPulseQueryLifecycle::new(owner, initial);
    let before = lifecycle.owner.read_status().unwrap();
    lifecycle.state = super::PlatformPulseQueryOwnerState::Indeterminate;
    let (lifecycle, denial) = match lifecycle.close() {
        Err(refused) => refused,
        Ok(_) => panic!("indeterminate product state cannot claim clean shutdown"),
    };
    let lifecycle = *lifecycle;
    assert!(matches!(denial, PlatformPulseQueryLifecycleDenial::IndeterminateAtClose));
    assert_eq!(lifecycle.owner.read_status().unwrap().value(), before.value());
    assert!(matches!(lifecycle.state, super::PlatformPulseQueryOwnerState::Indeterminate));
}
