//! Source binding validates admission authority before retention or mutation.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenialKind as Authorization,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::domain_computation::primary_graph::fact_verification_observation::fact_verification_on_this_thread_for_test as fact_work;
use crate::domain_computation::primary_graph::WorthQuerySourceExpectationDenialKind as Denial;
use crate::facade::primary_graph::WorthQueryExecutionObservation;
use crate::test_read_stops::{
    charged_prefixes_for_test, consult_count_for_test, observe_consults_for_test,
};
use std::{num::NonZeroUsize, time::Duration};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;

#[derive(Clone, Copy)]
enum Fault {
    None,
    Seal,
    Cancelled,
    Input,
}
fn check(fault: Fault, pending_entry: bool) {
    let source = &installation::install::<installation::OptionalSchema>(false);
    type Schema = installation::OptionalSchema;
    let cancellation =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let request =
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
    let external = primary_graph::tests::fixture::authenticate_external(
        source.installed_schema(),
        "alice",
        Duration::from_secs(60),
        &request,
    );
    let input = Input {
        scope_key: installation::ANCHOR.to_owned(),
        incomplete: false,
    };
    source
        .with_application_advancement(&request, |phase| {
            let selected = source
                .on_branch(source.current_world())
                .select(&phase)
                .unwrap();
            let principal_binding = source
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    installation::ANCHOR.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = source
                .installed_schema()
                .installed_operation(OrdinaryProduce::reference())
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
            let query = source
                .installed_schema()
                .installed_query_binding::<PlanarReadBinding<Schema>>()
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected
                .admit_application_query(
                    &phase,
                    query.query(),
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(NonZeroUsize::new(1).unwrap(), &request),
                )
                .unwrap();
            let (_, observed) = source
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source()
                .into_single_source()
                .unwrap();
            let pending = if pending_entry {
                Some(
                    source
                        .bind_application_source_expectation::<OrdinaryBinding<Schema>, _>(
                            &phase,
                            &mut admission,
                            observed.clone(),
                            &input,
                        )
                        .expect("genuine source binding reaches the pending phase"),
                )
            } else {
                None
            };
            match fault {
                Fault::None => {}
                Fault::Seal => {
                    let donor = source
                        .installed_schema()
                        .installed_operation(Produce::reference())
                        .unwrap();
                    admission.forge_installed_seal_for_test(donor.installed_view());
                }
                Fault::Cancelled => cancellation.cancel(),
                Fault::Input => admission.forge_tag_input_type_for_test::<u8>(),
            }
            let partition = admission.source_partition_identity();
            let _observation = WorthQueryExecutionObservation::install_on_this_thread();
            fact_work();
            let consults = observe_consults_for_test(|_| {});
            let outcome = match pending {
                Some(pending) => pending.consume_into(source, &phase, &mut admission),
                None => source
                    .bind_application_source_expectation::<OrdinaryBinding<Schema>, _>(
                        &phase,
                        &mut admission,
                        observed,
                        &input,
                    )
                    .map(drop),
            };
            let expected = match fault {
                Fault::None => {
                    outcome.expect("the genuine fresh source is admitted");
                    return;
                }
                Fault::Seal => Authorization::Refused(Refusal::StaleInstalledOperation),
                Fault::Cancelled => Authorization::Stopped(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
                )),
                Fault::Input => Authorization::Refused(Refusal::InputNotAdmitted),
            };
            let denial = outcome.unwrap_err();
            assert_eq!(denial.kind(), Denial::AdmissionAuthorization);
            assert_eq!(denial.authorization_denial().unwrap().kind(), expected);
            assert_eq!(admission.source_partition_identity(), partition);
            assert_eq!(fact_work().source_admission_visits, 0);
            assert_eq!(consult_count_for_test(&consults), 0);
            assert!(charged_prefixes_for_test(&consults).is_empty());
        })
        .unwrap();
}
#[test]
fn fresh_source_admission_reaches_binding_and_pending_consumption() {
    for pending in [false, true] {
        check(Fault::None, pending);
    }
}
#[test]
fn foreign_installed_seal_is_stale_at_source_binding_before_work() {
    check(Fault::Seal, false);
}
#[test]
fn foreign_installed_seal_is_stale_at_pending_source_consumption_before_work() {
    check(Fault::Seal, true);
}
#[test]
fn source_binding_checks_type_and_lapse_before_work() {
    for pending in [false, true] {
        check(Fault::Input, pending);
        check(Fault::Cancelled, pending);
    }
}
