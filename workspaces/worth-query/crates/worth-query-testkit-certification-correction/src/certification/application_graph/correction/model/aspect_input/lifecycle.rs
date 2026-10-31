//! Cancellation, expiry and revoked current authority cannot expose retained causes.
use super::super::{
    authentication::*,
    causal_inspection::{independent::retained_query, journey, output_query},
    host,
};
use super::{
    authoring::{Action, Disclosure as SourceDisclosure},
    courts::perform,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::{application_entry::*, causal_inspection::*, primary_graph::*};
pub fn removed_parent_rechecks_current_read_cancellation_and_expiry() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let _ = perform(
        &application,
        Action::Set {
            bias: 4,
            sibling: 40,
        },
        "aspect-lifecycle-present",
        1,
    );
    let (source, _, read) = perform(&application, Action::Remove, "aspect-lifecycle-removed", 1);
    let (after, _) = retained_query::read(&application, &principal, &scope, &read);
    drop(read);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    let [qm, qd, sm, sd] =
        std::array::from_fn(|_| journey::access(&application, &principal, &scope));
    cancellation.cancel();
    let denial = application
        .runtime()
        .request(&principal, &cancelled)
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source)
        .reference_only()
        .execute_governed(qm, qd, sm, sd)
        .err()
        .expect("actual cancellation refuses before source discovery");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    assert!(denial.producer_connection_refusal().is_none());
    drop(denial);
    let expired = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
        WorthQueryCancellationSource::new().token(),
    );
    let [qm, qd, sm, sd] =
        std::array::from_fn(|_| journey::access(&application, &principal, &scope));
    let denial = application
        .runtime()
        .request(&principal, &expired)
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source)
        .reference_only()
        .execute_governed(qm, qd, sm, sd)
        .err()
        .expect("actual expiry refuses before source discovery");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    assert!(denial.producer_connection_refusal().is_none());
    drop(denial);
    let [qm, qd, sm, sd] =
        std::array::from_fn(|_| journey::access(&application, &principal, &scope));
    let fresh = application.runtime().request(&principal, &scope);
    fresh
        .advanced()
        .mutate(super::super::principal_status::DisablePrincipal { identity: 1 })
        .without_source()
        .idempotency(&"independent-disable".to_owned())
        .execute_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let denial = fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source)
        .reference_only()
        .execute_governed(qm, qd, sm, sd)
        .err()
        .expect("current READ cannot be replaced by old capabilities");
    assert!(denial.producer_connection_refusal().is_none());
    drop(denial);
    assert_eq!(readers.observe().active(), baseline);
    after.close();
}
