//! Cancellation retained by an actual source binding reaches Publication recovery.
use super::{host, model, producer, query, schema::*};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryRequiredOutputPreparationDenial,
        WorthQueryRequiredOutputRecoveryPosture,
    },
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryPrincipalResolutionMode, WorthQueryProviderSessionControlStopKind as Control,
        WorthQuerySourceExpectationDenialKind,
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::authenticate_operator;

#[test]
fn entry_custody_cancelled_source_dependency_keeps_retryable_recovery() {
    let host = host::install(model::authored_value()).unwrap();
    let runtime = host.runtime();
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let external = authenticate_operator(host.installed_schema(), &request);
    let result = runtime
        .request(&external, &request)
        .query(query::ReadValue {
            identity: "authored-nested".into(),
        })
        .execute()
        .unwrap();
    let source = result.observed_sources()[0].clone();
    let cause = runtime
        .with_application_advancement(&request, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = host
                .installed_schema()
                .installed_mutation_binding::<producer::ProduceBinding>()
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    binding.principal_binding(),
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    "authored-nested".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = host
                .installed_schema()
                .installed_operation(producer::Produce::reference())
                .unwrap();
            let mut admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            cancellation.cancel();
            match runtime.bind_application_source_expectation::<producer::ProduceBinding, _>(
                &phase,
                &mut admission,
                source,
                &producer::Input {
                    identity: "authored-nested".into(),
                },
            ) {
                Err(cause) => cause,
                Ok(_) => panic!("the actual cancelled source entry must refuse"),
            }
        })
        .unwrap();
    assert_eq!(
        cause.kind(),
        WorthQuerySourceExpectationDenialKind::AdmissionAuthorization
    );
    assert_eq!(
        cause.authorization_denial().unwrap().kind(),
        Authorization::Stopped(Err(Control::Cancelled))
    );
    let denial = WorthQueryRequiredOutputPreparationDenial::from(cause);
    assert!(matches!(
        denial,
        WorthQueryRequiredOutputPreparationDenial::Dependencies(_)
    ));
    assert_eq!(
        denial.recovery_posture(),
        WorthQueryRequiredOutputRecoveryPosture::Retryable
    );
}
