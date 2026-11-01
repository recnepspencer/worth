//! Real public checkout drives the document's original SDK output tree.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryHistoryNavigationIntent;
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryPageLimits, WorthQueryHistoryRedoChoice,
};
use worth_query_topology_entry::PlanarHistoryNavigation;
mod authority;
mod checkout;
mod cutover;
mod interruption;
mod owner_discovery;
mod panic_disposal;
mod producer_admission;
mod profile;
mod publication;
type Outputs = WorthQueryHistoryWithOutputs<ConsumerSchema, HistoryProgram, HistoryRoot>;
type OutputDocument<'a> = WorthQueryApplicationHistoryDocument<
    'a,
    ConsumerSchema,
    History,
    OrdinaryHistoryPayload,
    Outputs,
>;

pub(super) fn navigate<P, O>(
    document: &WorthQueryApplicationHistoryDocument<
        '_,
        ConsumerSchema,
        History,
        OrdinaryHistoryPayload,
        O,
    >,
    app: &installation::ConsumerRuntime<P>,
    node: WorthQueryHistoryNodeId,
    key: u128,
) -> WorthQueryHistoryNavigationCompletion
where
    P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>,
    O: WorthQueryHistoryOutputsMode<ConsumerSchema, History>,
{
    let (_, mut advance) = with_request(app, |request| {
        checkout::start(document, request, node, key)
            .unwrap_or_else(|cause| panic!("new checkout click: {cause:?}"))
    });
    for _ in 0..96 {
        match advance {
            WorthQueryHistoryNavigationAdvance::Pending(next) => {
                advance = with_request(app, |request| next.advance(request));
            }
            WorthQueryHistoryNavigationAdvance::Selected(done) => {
                assert_eq!(done.selection().step().id(), node);
                return done;
            }
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("public output checkout: {cause:?}")
            }
        }
    }
    panic!("the finite original output tree did not select")
}

fn edit<'a>(
    document: OutputDocument<'a>,
    app: &Runtime,
    y: u64,
    key: u64,
) -> (OutputDocument<'a>, WorthQueryHistoryNodeId) {
    let mut edit = with_request(app, |request| {
        document
            .edit(PlanarInitialAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(y),
            })
            .idempotency_owned(request, key)
            .unwrap_or_else(|_| panic!("capture typed document edit"))
            .retain_in_program::<HistoryProgram, HistoryAction>(app)
            .unwrap_or_else(|(_, cause)| panic!("retain typed edit: {cause:?}"))
    });
    for _ in 0..96 {
        if edit.posture() == WorthQueryHistoryEditPosture::Selected {
            let finish = with_request(app, |request| {
                edit.finish(request)
                    .unwrap_or_else(|(_, cause)| panic!("finish typed edit: {cause:?}"))
            });
            let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = finish.outcome
            else {
                panic!("record a new future")
            };
            drop(delivery);
            return (finish.document, step.id());
        }
        edit = match with_request(app, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("typed edit: {denial:?}")
            }
        };
    }
    panic!("typed document edit did not select: {:?}", edit.posture())
}

#[test]
fn checkout_with_required_outputs_preserves_both_futures_and_the_document_mode() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, HistoryProgram>::begin()
        .with_schema(&ConsumerSchema::declaration().unwrap())
        .validated_program()
        .unwrap();
    // Root, two attached documents, three authored steps, the new future and
    // five navigation contexts are genuine live owners in this finite journey.
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
    warm(app, "anchor-a");
    warm(app, "remote-a");
    let mut document = attach(app, "anchor-a");
    let sibling = attach(app, "remote-a");
    let untouched = with_request(app, |request| {
        sibling.read(request).unwrap().rows().to_vec()
    });
    let mut nodes = Vec::new();
    for (index, y) in [2, 5, 8].into_iter().enumerate() {
        let result = finish(
            to_selection(
                begin(document, app, "anchor-a", y, 0x9182_e100 + index as u64),
                app,
            ),
            app,
        );
        let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = result.outcome else {
            panic!("record original future")
        };
        nodes.push(step.id());
        drop(delivery);
        document = result.document;
    }
    let document = app
        .with_host_advancement(|phase| {
            with_request(app, |request| {
                document
                    .with_outputs::<HistoryProgram, HistoryRoot>(
                        &phase,
                        request,
                        app,
                        PlanarOutputDemand::new("anchor-a"),
                        Default::default(),
                    )
                    .unwrap_or_else(|(_, cause)| {
                        panic!("configure actual output profile: {cause:?}")
                    })
            })
        })
        .unwrap();
    navigate(&document, app, nodes[0], 0x9182_e200);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    let (document, other) = edit(document, app, 6, 0x9182_e300);
    assert_eq!(
        with_request(app, |request| document
            .current_step(request)
            .unwrap()
            .parent()),
        Some(nodes[0])
    );
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(7)
    );
    navigate(&document, app, nodes[2], 0x9182_e201);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(9)
    );
    navigate(&document, app, other, 0x9182_e202);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(7)
    );
    for (intent, node, expected_length, key) in [
        (
            WorthQueryHistoryNavigationIntent::UndoStep,
            nodes[0],
            3,
            0x9182_e203,
        ),
        (
            WorthQueryHistoryNavigationIntent::RedoTo(other),
            other,
            7,
            0x9182_e204,
        ),
    ] {
        let hint = with_request(app, |request| {
            document
                .redo_choices(
                    request,
                    WorthQueryHistoryPageLimits {
                        maximum_entries: std::num::NonZeroUsize::MIN,
                        maximum_examined_entries: std::num::NonZeroUsize::MIN,
                        maximum_owned_bytes: std::num::NonZeroUsize::new(16 * 1024).unwrap(),
                    },
                    None,
                )
                .unwrap()
        });
        assert_eq!(
            hint.choice(),
            match intent {
                WorthQueryHistoryNavigationIntent::UndoStep => WorthQueryHistoryRedoChoice::NoChild,
                WorthQueryHistoryNavigationIntent::RedoTo(_) =>
                    WorthQueryHistoryRedoChoice::ChoiceRequired,
                WorthQueryHistoryNavigationIntent::Checkout(_) => unreachable!(),
            }
        );
        let mut owner = with_request(app, |request| {
            let builder = document.navigate::<PlanarHistoryNavigation<ConsumerSchema>>();
            let target = match intent {
                WorthQueryHistoryNavigationIntent::UndoStep => builder.undo_step(),
                WorthQueryHistoryNavigationIntent::RedoTo(child) => builder.redo_to(child),
                WorthQueryHistoryNavigationIntent::Checkout(_) => unreachable!(),
            };
            target
                .expected_selection(document.selection(request).unwrap())
                .idempotency(key)
                .prepare(request)
                .unwrap()
        });
        let mut completed = None;
        for _ in 0..96 {
            owner = match with_request(app, |request| owner.advance(request)) {
                WorthQueryHistoryNavigationAdvance::Pending(owner) => owner,
                WorthQueryHistoryNavigationAdvance::Selected(done) => {
                    completed = Some(done);
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                    panic!("edge output navigation: {cause:?}")
                }
            };
        }
        assert_eq!(completed.unwrap().requested_node(), node);
        assert_eq!(
            with_request(app, |request| document.read(request).unwrap().rows()[0]
                .value),
            length(expected_length)
        );
    }
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
