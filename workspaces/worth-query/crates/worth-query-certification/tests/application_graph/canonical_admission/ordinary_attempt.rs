//! Ordinary public read attempts retain decoder refusal until fact completion.
use super::{host, operation::InputBinding, schema::*};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{
    primary_graph::*, worth_query_operation, worth_query_operation_reads,
};

worth_query_operation!(pub ObserveCanonical for AdmissionSchema, input InputBinding);
worth_query_operation_reads!(ObserveCanonical => [DocumentValueField]);

struct Observed {
    second_refused: bool,
    completed: Result<(), WorthQueryApplicationAttemptDenial>,
}

fn observe_twice(host: &host::Host) -> Observed {
    super::opaque::reset();
    let runtime = host.runtime();
    let request = request_scope();
    let external = authenticate_operator(host.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = host
                .installed_schema()
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    "root".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = host
                .installed_schema()
                .installed_operation(ObserveCanonical::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let mut attempt = runtime
                .begin_application_read_attempt(&execution, admission)
                .unwrap();
            let first = attempt
                .observe_field(&execution, &root, DocumentValueField::reference())
                .unwrap();
            assert_eq!(
                first.0,
                super::value(),
                "ordinary observations decode the supported nondefault source"
            );
            let second_refused =
                match attempt.observe_field(&execution, &root, DocumentValueField::reference()) {
                    Ok(value) => {
                        assert_eq!(value.0, super::value());
                        false
                    }
                    Err(_) => true, // Deliberately ignore the read error and try to complete facts.
                };
            let completed = attempt
                .complete(
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .map(drop);
            Observed {
                second_refused,
                completed,
            }
        })
        .expect("ordinary read attempt borrows the real request advancement")
}

#[test]
fn ordinary_attempt_refusal_cannot_be_swallowed_before_fact_completion() {
    let funded = host::install(super::FUNDED_WORK);
    let positive = observe_twice(&funded);
    assert!(!positive.second_refused);
    positive
        .completed
        .expect("sufficient admitted ordinary observations complete real facts");
    assert_eq!(super::opaque::ignored_refusals(), 0);
    drop(funded);

    let bounded = host::install(super::ONE_VALUE_WORK);
    let refused = observe_twice(&bounded);
    assert!(
        refused.second_refused,
        "the cumulative second observation must refuse under the original request"
    );
    assert!(
        super::opaque::ignored_refusals() > 0,
        "the nested seed must have swallowed an actual native admission refusal"
    );
    let denial = refused
        .completed
        .expect_err("ignored observation failure cannot seal completed facts");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::ValueDecode(
            WorthQueryValueDecodeDenial::Checkpoint(worth_execution::MapKernelStop::WorkCeiling)
        )
    );
    assert_eq!(
        host::read(&bounded, "root").value,
        super::value(),
        "a fresh single-decode request is not poisoned"
    );
    let checkpoint = host::capture(&bounded);
    drop(bounded);
    let restored = host::restore(checkpoint, super::FUNDED_WORK);
    let retried = observe_twice(&restored);
    assert!(!retried.second_refused);
    retried
        .completed
        .expect("fresh sufficient current-format recovery completes ordinary facts");
    assert_eq!(host::read(&restored, "root").counter, 0);
}
