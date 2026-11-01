//! Current NAV is independent of READ, metadata and the retained historical grant.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind as Authorization,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_topology_entry::SetPlanarHistoryStatus;

pub(super) fn grant(app: &Runtime, branch: WorthQueryProductBranch, active: bool, key: u64) {
    let outcome = app
        .with_host_advancement(|phase| {
            with_branch_request(app, branch, |request| {
                request
                    .advanced()
                    .mutate(SetPlanarHistoryStatus {
                        grant: "history-navigation:model-owner:anchor-a".into(),
                        active,
                    })
                    .without_source()
                    .idempotency(&key)
                    .execute_in_program_in_advancement(&phase, app)
            })
        })
        .unwrap()
        .unwrap();
    assert!(
        matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { ref result, .. } if result.active == active),
        "ordinary current grant mutation must commit: {outcome:?}"
    );
}

pub(super) fn refresh(document: &mut OutputDocument<'_>, app: &Runtime) {
    with_request(app, |request| {
        document
            .refresh_context(
                request,
                WorthQueryHistoryContextRefreshLimits {
                    maximum_commits: std::num::NonZeroUsize::new(16).unwrap(),
                    maximum_records: std::num::NonZeroUsize::new(1024).unwrap(),
                    maximum_evidence_bytes: std::num::NonZeroU64::new(1_048_576).unwrap(),
                },
            )
            .unwrap()
    });
}

#[test]
fn output_checkout_requires_current_navigation_even_when_read_and_historical_grants_remain_lawful()
{
    let world = install();
    let app = &world.application;
    let (mut document, target) = producer_admission::configured_pair(app);
    let before = with_request(app, |request| document.selection(request).unwrap());
    let branch = before.physical_endpoint().product_branch();
    grant(app, branch, false, 0x9182_e900);
    refresh(&mut document, app);
    let expected = with_request(app, |request| document.selection(request).unwrap());
    assert_eq!(
        expected.step(),
        before.step(),
        "grant administration authors no model history step"
    );
    assert_ne!(expected.physical_endpoint(), before.physical_endpoint());
    assert_eq!(
        with_request(app, |request| document.current_step(request).unwrap()),
        *before.step()
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    let cause = with_request(app, |request| {
        document
            .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
            .checkout(target)
            .expected_selection(expected.clone())
            .idempotency(0x9182_e902)
            .prepare(request)
            .unwrap_err()
    });
    let WorthQueryHistoryNavigationAdvanceDenial::Outputs(
        WorthQueryHistoryOutputDenial::Navigation(
            WorthQueryHistoryNavigationDenial::Authorization(cause),
        ),
    ) = cause
    else {
        panic!("only current NAV was revoked: {cause:?}")
    };
    assert_eq!(
        cause.kind(),
        Authorization::Refused(Refusal::CapabilityAuthorizationMissing),
        "{cause:?}"
    );
    assert_eq!(cause.subject(), "NavigatePlanar");
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        expected
    );
    // The lawful twin restores the actual C grant; the unchanged historical
    // source alone was never fresh authority for the failed attempt.
    grant(app, branch, true, 0x9182_e903);
    refresh(&mut document, app);
    let done = navigate(&document, app, target, 0x9182_e902);
    assert_eq!(done.requested_node(), target);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
