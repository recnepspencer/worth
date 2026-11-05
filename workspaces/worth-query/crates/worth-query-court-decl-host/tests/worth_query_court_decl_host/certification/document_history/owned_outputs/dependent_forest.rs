//! A later owning producer can invalidate an earlier dependent family's input.
//! UpstreamOf is an ordinary reference, not declared ownership: Native current
//! publication custody alone proves that the generated Member belongs to Model.
use super::*;
type Document<'a> = WorthQueryApplicationHistoryDocument<
    'a,
    Schema,
    History,
    OrdinaryHistoryPayload,
    WorthQueryHistoryWithOutputTrees<Schema>,
>;

fn edited(
    host: &world::Host,
) -> (
    world::Document<'_>,
    WorthQueryHistoryNodeId,
    WorthQueryHistorySelection,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    upstream::settle(host, &request, "a");
    downstream::settle(host, &request, "upstream:a");
    let document = world::attach(host);
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .idempotency_owned(&request, "dependent-model-edit".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let step = document.current_step(&request).unwrap().id();
    let selection = document.selection(&request).unwrap();
    (document, step, selection)
}

fn configured<'a>(
    host: &'a world::Host,
    document: world::Document<'a>,
    upstream_first: bool,
) -> Document<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    host.with_host_advancement(|phase| {
        if upstream_first {
            document
                .with_output_trees::<program::Program, program::UpstreamTree>(
                    &phase,
                    &request,
                    host,
                    upstream::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
                .with_output_tree::<program::Program, program::DownstreamTree>(
                    &phase,
                    &request,
                    host,
                    downstream::Demand("upstream:a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        } else {
            document
                .with_output_trees::<program::Program, program::DownstreamTree>(
                    &phase,
                    &request,
                    host,
                    downstream::Demand("upstream:a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
                .with_output_tree::<program::Program, program::UpstreamTree>(
                    &phase,
                    &request,
                    host,
                    upstream::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        }
    })
    .unwrap()
}

fn values(
    host: &world::Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
) -> (query::Row, query::Row) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(branch);
    (
        request
            .query(query::member_public::Read("upstream:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
        request
            .query(query::child_output::Read("downstream:upstream:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
    )
}

fn exercise(host: &world::Host, upstream_first: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (document, step, selection) = edited(host);
    let before = values(host, selection.physical_endpoint().product_branch());
    assert_eq!((before.0.value, before.1.value), (5, 500));
    let document = configured(host, document, upstream_first);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = courts::checkout(&document, &request, step, selection.clone(), 0xf057_05);
    if upstream_first {
        let selected = outcome.unwrap_or_else(|cause| panic!("ordered dependent trees: {cause:?}"));
        let after = values(host, selected.physical_endpoint().product_branch());
        assert_eq!((after.0.value, after.1.value), (10, 1000));
        assert_eq!(
            (after.0.native_id, after.1.native_id),
            (before.0.native_id, before.1.native_id)
        );
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 2
        );
    } else {
        let denial = outcome.expect_err(
            "the later upstream publication invalidates the earlier dependent settlement",
        );
        assert!(
            matches!(
                denial,
                (
                    "advance",
                    WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                        WorthQueryHistoryOutputDenial::GraphQualification(
                            WorthQueryHistoryOutputQualificationDenial::Original(
                                WorthQueryOriginalReadOutputDenial::Currentness(_)
                            )
                        )
                    )
                )
            ),
            "{denial:?}"
        );
        assert_eq!(
            document.selection(&request).unwrap(),
            selection,
            "no adoption after a stale family"
        );
        assert_eq!(
            values(host, selection.physical_endpoint().product_branch()),
            before
        );
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 1,
            "the earlier dependent was initially healthy; only the later upstream ran"
        );
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn dependent_families_observe_the_preceding_fallback_on_the_same_private_candidate() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install(), true);
}
#[test]
fn a_later_fallback_cannot_adopt_an_earlier_now_stale_family() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install(), false);
}
#[test]
fn worker_only_dependent_families_observe_the_preceding_fallback() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install_worker_only(), true);
}
#[test]
fn worker_only_later_fallback_cannot_adopt_an_earlier_now_stale_family() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install_worker_only(), false);
}
