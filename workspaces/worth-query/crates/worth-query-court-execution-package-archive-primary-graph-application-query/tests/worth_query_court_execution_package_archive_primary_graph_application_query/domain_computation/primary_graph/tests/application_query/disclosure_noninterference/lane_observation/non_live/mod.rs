use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::governed_live_account_parameters;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::capture_lane;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::one;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::ContinuationObservation;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::GovernedObservationContext;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::GraphWorkOccurrences;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::LaneObservation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryResumeControls;

pub fn observe_one_shot(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    context: &GovernedObservationContext<'_>,
    occurrences: &mut GraphWorkOccurrences,
) -> LaneObservation {
    let capability = admit_touch_account_capability(
        &(phase_custody).test_host_request(),
        context.world,
        context.principal,
        context.request,
    )
    .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(context.principal, context.account);
    let plan = context
        .world
        .selected_product(&(phase_custody).test_host_request())
        .admit_governed_application_query(
            &phase_custody.phase_for_application(&context.world.application),
            context.query,
            &access,
            capability,
            governed_live_account_parameters("account-1"),
            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                one(),
                context.request,
            ),
        )
        .unwrap();
    let result = context
        .world
        .application
        .execute_application_query_one_shot(
            &phase_custody.phase_for_application(&context.world.application),
            plan,
        )
        .unwrap();
    capture_lane(result.rows().to_vec(), result.receipt(), occurrences)
}

pub fn observe_continuation(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    context: &GovernedObservationContext<'_>,
    occurrences: &mut GraphWorkOccurrences,
) -> ContinuationObservation {
    let capability = admit_touch_account_capability(
        &(phase_custody).test_host_request(),
        context.world,
        context.principal,
        context.request,
    )
    .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(context.principal, context.account);
    let plan = context
        .world
        .selected_product(&(phase_custody).test_host_request())
        .admit_governed_application_query_continuation(
            &phase_custody.phase_for_application(&context.world.application),
            context.query,
            &access,
            capability,
            governed_live_account_parameters("account-1"),
            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                one(),
                context.request,
            ),
        )
        .unwrap();
    let page = context
        .world
        .application
        .execute_application_query_continuation_page(
            &phase_custody.phase_for_application(&context.world.application),
            plan,
        )
        .unwrap();
    let (rows, continuation, receipt) = page.into_parts();
    let first = capture_lane(rows, &receipt, occurrences);
    let continuation = continuation.expect("the first activity page must continue");
    let first_next_page_ordinal = continuation.page_ordinal();

    let fresh = admit_touch_account_capability(
        &(phase_custody).test_host_request(),
        context.world,
        context.principal,
        context.request,
    )
    .unwrap();
    let plan = context
        .world
        .application
        .readmit_governed_application_query_continuation(
            &phase_custody.phase_for_application(&context.world.application),
            context.query,
            &access,
            fresh,
            governed_live_account_parameters("account-1"),
            continuation,
            WorthQueryApplicationQueryResumeControls::new(one(), context.request),
        )
        .unwrap();
    let page = context
        .world
        .application
        .execute_application_query_continuation_page(
            &phase_custody.phase_for_application(&context.world.application),
            plan,
        )
        .unwrap();
    let (rows, continuation, receipt) = page.into_parts();
    ContinuationObservation {
        first,
        first_next_page_ordinal,
        second: capture_lane(rows, &receipt, occurrences),
        second_has_continuation: continuation.is_some(),
    }
}
