//! A stopped request enters no owner call and leaves the dirty demand retryable.
use super::*;
use std::time::{Duration, Instant};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::WorthQueryApplicationOutputDemandDenial,
};

pub(in super::super) fn before_decision(application: &Application) {
    let (scope, principal) = authenticate(application);
    let request = application.request(&principal, &scope);
    let mut handle = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram, RegionConnection>(application)
        .unwrap();
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let stopped_scope = authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    );
    let stopped_request = application.request(&principal, &stopped_scope);
    room().clear();
    published_states();
    cancellation.cancel();
    let stopped = handle.advance(&stopped_request);
    assert!(
        matches!(&stopped, Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
        if denial.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(
            WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled)
        )),
        "the request's cancellation settles in one advance: {:?}",
        stopped.as_ref().err()
    );
    assert!(room().is_empty(), "a canceled advance contacts no producer");
    assert_eq!(take_calls(), OwnerCalls::default(), "canceled call law");
    assert!(
        published_states().is_empty(),
        "cancellation publishes nothing"
    );
    // The alphabet's ordinary demand follows on an admitted request, proving
    // the cancellation neither consumes the edit nor replaces retained state.
}
