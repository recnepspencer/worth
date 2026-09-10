use super::super::super::super::{
    fixture::{
        admit_touch_account_capability, governed_live_account_parameters, Account,
        AccountSummaryParameters, Activity, GovernedLiveAccountActivityCause,
        GovernedLiveAccountActivityQuery, GovernedLiveAccountActivityResult,
    },
    live_delivery_support::commit_live_activity_with_label,
};
use super::{
    capture_lane, installed_governed_query, resolve_account, GovernedObservationContext,
    GraphWorkOccurrences, LaneObservation, StableReadCompletionObservation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveControls,
    WorthQueryApplicationLiveOutcome,
};

pub(super) fn observe(
    context: &GovernedObservationContext<'_>,
    label: &str,
    occurrences: &mut GraphWorkOccurrences,
) -> (LaneObservation, u64, StableReadCompletionObservation) {
    let query = installed_governed_query(context.world);
    let account = resolve_account(context.world, context.request);
    let capability =
        admit_touch_account_capability(context.world, context.principal, context.request).unwrap();
    let mut lease = context
        .world
        .selected_product()
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
        >(
            query,
            context.principal,
            account,
            capability,
            governed_live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(
                context.request.clone(),
                4,
                16,
                2_048,
            )
            .unwrap(),
        )
        .unwrap();
    let committed =
        commit_live_activity_with_label(context.world, context.committer, context.request, label);
    let WorthQueryApplicationLiveOutcome::Delivered(update) = lease.poll() else {
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
