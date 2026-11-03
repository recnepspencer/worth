use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::governed_live_account_parameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Activity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityCause;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedLiveAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::live_delivery_support::commit_live_activity_with_label;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::capture_lane;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::installed_governed_query;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::resolve_account;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::GovernedObservationContext;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::GraphWorkOccurrences;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::LaneObservation;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::receipt_observation::StableReadCompletionObservation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveCloseOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveControls;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveOutcome;

pub fn observe(
    context: &GovernedObservationContext<'_>,
    label: &str,
    occurrences: &mut GraphWorkOccurrences,
) -> (LaneObservation, u64, StableReadCompletionObservation) {
    let mut lease = context
        .world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let phase_custody = &custody;
            let query = installed_governed_query(context.world);
            let account = resolve_account(
                phase_custody,
                &phase_custody.borrowed_request_for_test(),
                context.world,
                context.request,
            );
            let capability = admit_touch_account_capability(
                &(phase_custody).test_host_request(),
                context.world,
                context.principal,
                context.request,
            )
            .unwrap();

            context
        .world
        .selected_product(&(phase_custody).test_host_request(), )
        .open_governed_application_query_live::<
            GovernedLiveAccountActivityQuery,
            AccountSummaryParameters,
            GovernedLiveAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            GovernedLiveAccountActivityCause,
            _,
            _,
            _,
        >(&phase_custody.phase_for_application(&context.world.application),
            query,
            context.principal,
            account,
            capability,
            governed_live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(context.request.clone(), 4, 16)
            .unwrap(),
        )
        .unwrap()
        })
        .expect("the fixture owns its live opening call");
    let committed =
        commit_live_activity_with_label(context.world, context.committer, context.request, label);
    let WorthQueryApplicationLiveOutcome::Delivered(update) = context
        .world
        .application
        .with_host_advancement(|phase| lease.next(&phase, context.principal, context.request))
        .expect("the fixture owns its delivery call")
    else {
        panic!("the symmetric live cause must produce one governed delivery");
    };
    assert_eq!(
        update.product_publication(),
        committed.committed_product_publication()
    );
    let live_commit_ordinal = update.product_publication().composite_commit().ordinal();
    let lane = capture_lane(vec![update.result().clone()], update.receipt(), occurrences);
    let WorthQueryApplicationLiveCloseOutcome::Completed(completion) = lease.close() else {
        panic!("the opening live graph-work session must close cleanly");
    };
    occurrences.capture(&completion);
    let close = StableReadCompletionObservation::capture(&completion);
    (lane, live_commit_ordinal, close)
}
