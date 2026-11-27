//! An envelope on a document with bound output trees records every tree in its one step.
use super::*;

mod owned_document;
mod property;
pub(crate) mod step_work;
mod stopped_request;

type Forest<'a> = Document<'a, WorthQueryHistoryWithOutputTrees<Schema>>;

/// The authored rows and both families' outputs at the document's head.
#[derive(Clone, Debug, PartialEq)]
struct Snapshot {
    model: u64,
    member: u64,
    outputs: (query::Row, query::Row),
}

fn snapshot<O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: &Document<'_, O>,
) -> Snapshot {
    let seen = observe(host, document);
    Snapshot {
        model: seen.model,
        member: seen.member,
        outputs: world::outputs(host, seen.branch),
    }
}

/// Settle both families on the installed world, then bind both trees.
fn attach_with_both_trees(host: &world::Host) -> Forest<'_> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(host, &request, "a");
    second::settle(host, &request, "a");
    courts::configure_all_trees(host, world::attach(host))
}

fn record_step<'d>(
    host: &world::Host,
    document: Forest<'d>,
    key: &'static str,
    model: u64,
    member: u64,
) -> (Forest<'d>, WorthQueryHistoryStep) {
    let stages = Stages { key, model, member };
    recorded(record(host, document, &request_scope(), stages, metadata(b"")))
}

enum Move {
    Undo,
    RedoTo(WorthQueryHistoryNodeId),
    Checkout(WorthQueryHistoryNodeId),
}

/// Navigate from the document's current selection until the move ends.
fn navigate(host: &world::Host, document: &Forest<'_>, to: Move, idempotency: u128) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let expected = document.selection(&request).unwrap();
    let builder = document.navigate::<navigation::Navigation>();
    let target = match to {
        Move::Undo => builder.undo_step(),
        Move::RedoTo(child) => builder.redo_to(child),
        Move::Checkout(node) => builder.checkout(node),
    };
    let mut navigation = target
        .expected_selection(expected)
        .idempotency(idempotency)
        .prepare(&request)
        .unwrap_or_else(|cause| panic!("prepare: {cause:?}"));
    for _ in 0..32 {
        navigation = match navigation.advance(&request) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            WorthQueryHistoryNavigationAdvance::Selected(_) => return,
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("navigation refused: {cause:?}")
            }
        };
    }
    panic!("bounded navigation must end");
}

#[test]
fn an_envelope_step_records_both_output_trees() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = attach_with_both_trees(&host);
    let before = observe(&host, &document);
    let original = snapshot(&host, &document);
    assert_eq!(
        (original.outputs.0.value, original.outputs.1.value),
        (10, 1001)
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (document, step) = record_step(&host, document, "both-trees", 2, 6);
    assert_eq!(step.parent(), Some(before.current));
    let after = observe(&host, &document);
    assert_eq!(after.current, step.id());
    assert_eq!(
        after.nodes.len(),
        before.nodes.len() + 1,
        "the envelope and both trees are one step"
    );
    let recorded = snapshot(&host, &document);
    assert_eq!((recorded.model, recorded.member), (2, 6));
    assert_eq!(
        (recorded.outputs.0.value, recorded.outputs.1.value),
        (20, 1002),
        "the step holds both families at the edited source"
    );
    assert_eq!(
        (recorded.outputs.0.native_id, recorded.outputs.1.native_id),
        (original.outputs.0.native_id, original.outputs.1.native_id)
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "each family's producer ran once for the step"
    );
    let settled = host.producer_contacts_on_this_thread_for_test();
    navigate(&host, &document, Move::Undo, 0x57e0_0001);
    assert_eq!(snapshot(&host, &document), original);
    navigate(&host, &document, Move::RedoTo(step.id()), 0x57e0_0002);
    assert_eq!(snapshot(&host, &document), recorded);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        settled,
        "both recorded trees restore without a producer"
    );
    close(&host, document);
}

#[test]
fn undo_branch_and_redo_restore_both_trees() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = attach_with_both_trees(&host);
    let (document, a) = record_step(&host, document, "step-a", 2, 6);
    let at_a = snapshot(&host, &document);
    let (document, b) = record_step(&host, document, "step-b", 3, 7);
    let at_b = snapshot(&host, &document);
    assert_eq!(b.parent(), Some(a.id()));
    navigate(&host, &document, Move::Undo, 0x57e0_0011);
    assert_eq!(snapshot(&host, &document), at_a);
    let (document, c) = record_step(&host, document, "step-c", 4, 8);
    let at_c = snapshot(&host, &document);
    assert_eq!(c.parent(), Some(a.id()), "the new step branches from A");
    assert_eq!((at_c.outputs.0.value, at_c.outputs.1.value), (40, 1004));
    let settled = host.producer_contacts_on_this_thread_for_test();
    navigate(&host, &document, Move::Undo, 0x57e0_0012);
    assert_eq!(snapshot(&host, &document), at_a);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let choices = document.redo_choices(&request, limits(), None).unwrap();
    assert_eq!(choices.choice(), WorthQueryHistoryRedoChoice::ChoiceRequired);
    let children: Vec<_> = choices
        .children()
        .rows()
        .iter()
        .map(|row| row.step().id())
        .collect();
    assert_eq!(children, [b.id(), c.id()]);
    drop(choices);
    navigate(&host, &document, Move::RedoTo(b.id()), 0x57e0_0013);
    assert_eq!(snapshot(&host, &document), at_b);
    navigate(&host, &document, Move::Checkout(c.id()), 0x57e0_0014);
    assert_eq!(snapshot(&host, &document), at_c);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        settled,
        "every recorded tree restores without a producer"
    );
    close(&host, document);
}

/// The second family is owned by the model but bound to no tree, so the step
/// would leave it stale. The envelope refuses and nothing is recorded.
#[test]
fn an_unbound_native_family_refuses_incomplete_and_leaves_the_document_intact() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let document = world::attach(&host);
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::FirstTree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let before = observe(&host, &document);
    let original = snapshot(&host, &document);
    let index = document.history_source_index_for_test();
    let selection = document.selection(&request).unwrap();
    let model = world::model(&host, before.branch).native_id;
    let stages = Stages {
        key: "unbound-family",
        model: 2,
        member: 6,
    };
    let (document, denial) = refused(record(&host, document, &scope, stages, metadata(b"")));
    assert!(
        matches!(
            denial,
            Denial::Outputs(WorthQueryHistoryOutputDenial::RecordingQualification(
                WorthQueryRecordingOutputQualificationDenial::Incomplete { model: subject }
            )) if subject == model
        ),
        "{denial:?}"
    );
    assert_eq!(observe(&host, &document), before);
    assert_eq!(snapshot(&host, &document), original);
    assert_eq!(document.selection(&request).unwrap(), selection);
    assert_eq!(
        document.history_source_index_for_test(),
        index,
        "the refusal releases its index entries"
    );
    // The refusal left no recording behind: the same key refuses the same way.
    let (document, again) = refused(record(&host, document, &scope, stages, metadata(b"")));
    assert!(
        matches!(
            again,
            Denial::Outputs(WorthQueryHistoryOutputDenial::RecordingQualification(
                WorthQueryRecordingOutputQualificationDenial::Incomplete { .. }
            ))
        ),
        "{again:?}"
    );
    assert_eq!(observe(&host, &document), before);
    close(&host, document);
}
