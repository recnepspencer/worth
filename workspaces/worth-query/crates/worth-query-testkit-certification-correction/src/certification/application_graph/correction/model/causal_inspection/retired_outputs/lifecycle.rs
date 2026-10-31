//! Real authored retirement, replacement and grant withdrawal under the surviving source.
use super::super::super::{authored_identity_model, schema::*};
use super::*;
use worth_query_host::facade::admission::authenticated_principal::*;

pub(super) fn retire(application: &host::OutputHost) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(authored_identity_model::Retire {
            key: "numeric".to_owned(),
            target: "observer".to_owned(),
            wrong_restore_key: false,
            collateral: None,
        })
        .without_source()
        .idempotency(&"retire-produced-observer".to_owned())
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "actual output retirement: {outcome:?}"
    );
}
pub(super) fn restore(application: &host::OutputHost, calculated: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(authored_identity_model::Restore {
            key: "numeric".to_owned(),
            name: "observer".to_owned(),
            primary: 17,
            secondary: 0,
            read_enabled: true,
            ceiling: 1000,
            calculated,
            collateral: None,
        })
        .without_source()
        .idempotency(&"replace-produced-observer".to_owned())
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "actual equal-byte replacement: {outcome:?}"
    );
}
pub(super) fn revoke_and_refuse(
    application: &host::OutputHost,
    principal: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
    after: &WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
) {
    let qm = journey::access(application, principal, scope);
    let qd = journey::access(application, principal, scope);
    let sm = journey::access(application, principal, scope);
    let sd = journey::access(application, principal, scope);
    let request = application.runtime().request(principal, scope);
    let withdrawn = request
        .advanced()
        .mutate(distinct_subjects::grant_status::Withdraw {
            grant: "inspection-grant".to_owned(),
        })
        .without_source()
        .idempotency(&"withdraw-retired-output-inspection".to_owned())
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            withdrawn,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "real grant withdrawal: {withdrawn:?}"
    );
    let denial = request
        .inspect::<query::Binding>(after)
        .why_observed()
        .caused_by::<SourceDisclosure>(source)
        .reference_only()
        .execute_governed(qm, qd, sm, sd)
        .err()
        .expect("retained cause evidence cannot replace current permission");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denial.producer_connection_refusal().is_none());
    assert!(denial.producer_output_refusal().is_none());
}
