//! A frozen discovery request carries no permission from its selected D.
use super::*;
fn exercise_selection(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: Host = world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request);
    let document = world::attach(&host);
    let before = document.current_step(&request).unwrap().id();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let document = record!(
        request,
        host,
        document,
        member_edit::Intent {
            key: "m".into(),
            value: 7
        },
        super::super::super::super::program::MemberEditAction,
        "record-c-without-discovery-grant"
    );
    let selection = document.selection(&request).unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let reads = source_projection::Observation::begin();
    let (_, denial) = super::super::super::super::courts::checkout(
        &document,
        &request,
        before,
        selection.clone(),
        0x0012_55c1,
    )
    .unwrap_err();
    assert_eq!(
        reads.discovery_reads(),
        0,
        "C authorization precedes any D discovery disclosure"
    );
    let WorthQueryHistoryNavigationAdvanceDenial::Outputs(
        WorthQueryHistoryOutputDenial::Preparation(
            WorthQueryRequiredOutputPreparationDenial::SourceQuery(
                WorthQueryApplicationRequestQueryDenial::Admission(cause),
            ),
        ),
    ) = denial
    else {
        panic!("wrong current-permission refusal: {denial:?}")
    };
    assert!(
        matches!(
            cause.kind(),
            WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_)
        ),
        "{cause:?}"
    );
    assert_eq!(
        reads.reads(),
        0,
        "no healthy root is opened to get a settlement"
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(document.selection(&request).unwrap(), selection);
    let at = host
        .request(&principal, &scope)
        .on_branch(selection.physical_endpoint().product_branch());
    assert_eq!(
        at.query(query::member_public::Read("m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        7
    );
    assert_eq!(
        at.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        30
    );
    assert_eq!(
        at.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1003
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn a_historical_recipe_uses_current_permission_before_discovery() {
    exercise_selection(false);
}
#[test]
fn worker_only_a_historical_recipe_uses_current_permission_before_discovery() {
    exercise_selection(true);
}
