//! Complete E includes both independently changed, genuinely required children.
use super::*;
use crate::application_program::historical::branching::{Action, Program, Root};

type App = installation::ConsumerRuntime<Program>;
type Edit<'a> = WorthQueryHistoryEdit<'a, ConsumerSchema, History, Original, Program, Action>;

fn record<'a>(
    document: Document<'a>,
    app: &App,
    y: u64,
    key: u64,
) -> (Document<'a>, WorthQueryHistoryNodeId) {
    let mut edit: Edit<'a> = with_request(app, |request| {
        document
            .edit(Original(PlanarInitialAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(y),
            }))
            .idempotency_owned(request, key)
            .unwrap_or_else(|_| panic!("capture two-child edit"))
            .retain_in_program::<Program, Action>(app)
            .unwrap_or_else(|(_, cause)| panic!("retain two-child edit: {cause:?}"))
    });
    for _ in 0..96 {
        if edit.posture() == WorthQueryHistoryEditPosture::Selected {
            let result = with_request(app, |request| edit.finish(request))
                .unwrap_or_else(|(_, cause)| panic!("finish two-child history: {cause:?}"));
            let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = result.outcome
            else {
                panic!("record an ordinary two-child edit")
            };
            drop(delivery);
            return (result.document, step.id());
        }
        if edit.posture() == WorthQueryHistoryEditPosture::AwaitingSelection {
            assert_eq!(
                edit.completed_source().unwrap().settlement().output_count(),
                2,
                "both original declared children must actually settle"
            );
        }
        edit = match with_request(app, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("two-child edit: {denial:?}")
            }
        };
    }
    panic!("the finite two-child history edit did not finish")
}

fn outside(app: &App, branch: WorthQueryProductBranch, body: &str, y: u64, key: u64) {
    let mut operation = with_branch_request(app, branch, |request| {
        request
            .mutate(Original(PlanarInitialAdjustment {
                scope_key: body.into(),
                replacement_y: length(y),
            }))
            .without_source()
            .idempotency_owned(key)
            .unwrap_or_else(|(_, _, cause)| panic!("outside input: {cause:?}"))
            .retain_operation::<Program, Action>(app)
            .unwrap_or_else(|(_, cause)| panic!("outside ownership: {cause:?}"))
    });
    for _ in 0..96 {
        match with_branch_request(app, branch, |request| operation.advance(request)) {
            WorthQueryOperationAdvance::Progress {
                operation: next, ..
            } => operation = next,
            WorthQueryOperationAdvance::Settled(settled) => {
                let delivery =
                    with_branch_request(app, branch, |request| settled.close_for_delivery(request))
                        .unwrap_or_else(|(_, cause)| panic!("outside delivery: {cause:?}"));
                drop(delivery);
                return;
            }
            _ => panic!("the actual outside-model operation must settle"),
        }
    }
    panic!("outside-model operation did not finish")
}

fn y(app: &App, branch: WorthQueryProductBranch, body: &str) -> u64 {
    with_branch_request(app, branch, |request| {
        worth_query_consumer_values::PositiveLength::get(
            &request
                .query(PlanarRead {
                    body_key: body.into(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .y,
        )
    })
}

#[test]
fn checkout_adopts_both_rebuilt_children_from_the_same_original_candidate_tree() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let program = ApplicationProgramAuthoring::<ConsumerSchema, Program>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap())
        .validated_program()
        .unwrap();
    let world = installation::install_selected_program_with_limits(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        false,
        crate::application_invariant_acceptance::resources::application_limits_with_branches(12),
        program,
    )
    .unwrap();
    let app = &world.application;
    let sibling = attach(app, "remote-a");
    let untouched = with_request(app, |request| {
        sibling.read(request).unwrap().rows().to_vec()
    });
    let (document, target) = record(attach(app, "anchor-a"), app, 2, 0x9182_e700);
    let (document, _) = record(document, app, 3, 0x9182_e701);
    let (mut document, latest) = record(document, app, 5, 0x9182_e702);
    let branch = with_request(app, |request| {
        document
            .selection(request)
            .unwrap()
            .physical_endpoint()
            .product_branch()
    });
    assert_eq!(y(app, branch, "final:remote-b"), 43);
    assert_eq!(y(app, branch, "final:anchor-b"), 3);
    // ScopeEntityRecord history owns anchor-a alone. Both actual child sources
    // are outside that model and retain these current values during checkout.
    outside(app, branch, "remote-b", 44, 0x9182_e710);
    outside(app, branch, "anchor-b", 4, 0x9182_e711);
    assert_eq!(y(app, branch, "final:remote-b"), 43);
    assert_eq!(y(app, branch, "final:anchor-b"), 3);
    with_request(app, |request| {
        document
            .refresh_context(
                request,
                WorthQueryHistoryContextRefreshLimits {
                    maximum_commits: std::num::NonZeroUsize::new(32).unwrap(),
                    maximum_records: std::num::NonZeroUsize::new(4096).unwrap(),
                    maximum_evidence_bytes: std::num::NonZeroU64::new(4_194_304).unwrap(),
                },
            )
            .unwrap()
    });
    let document = app
        .with_host_advancement(|phase| {
            with_request(app, |request| {
                document
                    .with_outputs::<Program, Root>(
                        &phase,
                        request,
                        app,
                        PlanarOutputDemand::new("anchor-a"),
                        Default::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("two-child profile: {cause:?}"))
            })
        })
        .unwrap();
    let expected = with_request(app, |request| document.selection(request).unwrap());
    let prepare = || {
        with_request(app, |request| {
            document
                .navigate::<worth_query_topology_entry::PlanarHistoryNavigation<ConsumerSchema>>()
                .checkout(target)
                .expected_selection(expected.clone())
                .idempotency(0x9182_e720)
                .prepare(request)
                .unwrap()
        })
    };
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let mut navigation = prepare();
    let mut dropped_candidate = None;
    for _ in 0..96 {
        match with_request(app, |request| navigation.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => {
                if app.producer_contacts_on_this_thread_for_test() > contacts {
                    dropped_candidate = next.candidate_branch_for_test();
                    drop(next);
                    break;
                }
                navigation = next;
            }
            _ => panic!("real output work must yield before selection"),
        }
    }
    let original_d = dropped_candidate.expect("drop after actual output handler work on D");
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        expected
    );
    // Only the frontend handle was lost. The document retains the original
    // candidate and output tree under the same immutable navigation intent.
    let mut navigation = prepare();
    assert_eq!(navigation.candidate_branch_for_test(), Some(original_d));
    let mut completed = None;
    for _ in 0..96 {
        match with_request(app, |request| navigation.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => {
                assert_eq!(next.candidate_branch_for_test(), Some(original_d));
                navigation = next;
            }
            WorthQueryHistoryNavigationAdvance::Selected(done) => {
                completed = Some(done);
                break;
            }
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("recover original two-child tree: {cause:?}")
            }
        }
    }
    let done = completed.expect("recovered original tree selects");
    assert_eq!(done.requested_node(), target);
    let selected = done.selection().physical_endpoint().product_branch();
    assert_eq!(selected, original_d);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    assert_eq!(y(app, selected, "remote-b"), 44);
    assert_eq!(y(app, selected, "anchor-b"), 4);
    assert_eq!(
        y(app, selected, "final:remote-b"),
        46,
        "omitting the late-discovered child leaves43"
    );
    assert_eq!(
        y(app, selected, "final:anchor-b"),
        6,
        "omitting the other required child leaves3"
    );
    let original_completion = done.selection().clone();
    drop(done);
    let later = super::super::navigation_outputs::navigate(&document, app, latest, 0x9182_e721);
    let later_selection = later.selection().clone();
    assert_ne!(
        later_selection.physical_endpoint().product_branch(),
        original_d
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    // Recreate the exact original request after navigating elsewhere. Its
    // cached outcome is observable without resetting the current head to D.
    let mut retry = prepare();
    let mut delivered = false;
    for _ in 0..96 {
        match with_request(app, |request| retry.advance(request)) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => retry = next,
            WorthQueryHistoryNavigationAdvance::Selected(original) => {
                assert_eq!(original.selection(), &original_completion);
                delivered = true;
                break;
            }
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("retry original completion after later navigation: {cause:?}")
            }
        }
    }
    assert!(delivered);
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        later_selection
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    assert_eq!(
        with_request(app, |request| sibling
            .read(request)
            .unwrap()
            .rows()
            .to_vec()),
        untouched
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    close(sibling, app);
}
