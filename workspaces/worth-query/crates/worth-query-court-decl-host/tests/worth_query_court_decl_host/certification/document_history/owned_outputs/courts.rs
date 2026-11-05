//! Source-scope-owned producer outputs across a recorded owned-model history edit.
use super::*;
use family::Request;
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_relational::facade::{
    mvcc::{
        RelationalOwnedGraphSelectionPreparationError as GraphPreparation, TransactionCommitError,
    },
    publication::AuthoredRetirementRefusal,
};

struct Edited<'a> {
    document: world::Document<'a>,
    before: WorthQueryHistoryNodeId,
    branch: WorthQueryProductBranch,
    selection: WorthQueryHistorySelection,
    outputs: (query::Row, query::Row),
}

/// Settle both families on the current world (Initial), attach the model
/// history and record one model edit. With `resettle`, settle both families
/// again on the edited branch (Preserve). Counts actual producer handler entries.
fn edited<'a>(host: &'a world::Host, request: &Request<'_, '_, '_>, resettle: bool) -> Edited<'a> {
    let contacts = host.producer_contacts_on_this_thread_for_test();
    first::settle(host, request, "a");
    second::settle(host, request, "a");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "each family's Initial producer ran exactly once"
    );
    let initial = world::outputs(host, host.current_world());
    assert_eq!((initial.0.value, initial.1.value), (10, 1001));
    let document = world::attach(host);
    let before = document.current_step(request).unwrap().id();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .idempotency_owned(request, "edit-model".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    let WorthQueryHistoryEditOutcome::Recorded { step, .. } = outcome else {
        panic!("the model edit must record")
    };
    assert_eq!(step.label(), Some("Edit model"));
    let selection = document.selection(request).unwrap();
    let branch = selection.physical_endpoint().product_branch();
    assert_eq!(world::model(host, branch).value, 2);
    if !resettle {
        let outputs = world::outputs(host, branch);
        assert_eq!(
            outputs, initial,
            "outputs are untouched until settled again"
        );
        return Edited {
            document,
            before,
            branch,
            selection,
            outputs,
        };
    }
    let scope = request_scope();
    let operator = authenticate_operator(host.installed_schema(), &scope);
    let on_branch = host.request(&operator, &scope).on_branch(branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    first::settle(host, &on_branch, "a");
    second::settle(host, &on_branch, "a");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "each family's Preserve producer ran exactly once"
    );
    let outputs = world::outputs(host, branch);
    assert_eq!((outputs.0.value, outputs.1.value), (20, 1002));
    assert_eq!(
        (outputs.0.native_id, outputs.1.native_id),
        (initial.0.native_id, initial.1.native_id),
        "Preserve keeps each family's original output identity"
    );
    Edited {
        document,
        before,
        branch,
        selection,
        outputs,
    }
}

#[test]
fn two_families_settle_after_a_recorded_model_edit() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let Edited { document, .. } = edited(&host, &request, true);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

/// Prepare and advance a checkout of the pre-edit step until it ends.
fn checkout(
    document: &world::Document<'_>,
    request: &Request<'_, '_, '_>,
    before: WorthQueryHistoryNodeId,
    expected: WorthQueryHistorySelection,
) -> Result<WorthQueryHistorySelection, (&'static str, WorthQueryHistoryNavigationAdvanceDenial)> {
    let mut navigation = document
        .navigate::<navigation::Navigation>()
        .checkout(before)
        .expected_selection(expected)
        .idempotency(0x5356_0001)
        .prepare(request)
        .map_err(|cause| ("prepare", cause))?;
    for _ in 0..32 {
        navigation = match navigation.advance(request) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            WorthQueryHistoryNavigationAdvance::Selected(done) => {
                return Ok(done.selection().clone())
            }
            WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                owner
                    .discard(request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                return Err(("advance", cause));
            }
        };
    }
    panic!("bounded checkout must end");
}

/// Outputs settled before attach; the edit alone is recorded. Native refuses
/// the owned-graph checkout at `prepare` with the model's owned producer
/// publication (`OwnedPublicationRequiresCascade`), because Query does not yet
/// pass HistoricalSelection output custody.
#[test]
fn owned_graph_checkout_over_producer_publications_refuses_typed_today() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let Edited {
        document,
        before,
        branch,
        selection,
        outputs,
    } = edited(&host, &request, false);
    let model = world::model(&host, branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let refusal = checkout(&document, &request, before, selection.clone())
        .expect_err("checkout over owned producer publications must refuse today");
    let (
        "prepare",
        WorthQueryHistoryNavigationAdvanceDenial::Navigation(
            WorthQueryHistoryNavigationDenial::GraphPreparation(GraphPreparation::Preparation(
                TransactionCommitError::Publication { error, .. },
            )),
        ),
    ) = &refusal
    else {
        panic!("expected Native owned-publication refusal at prepare: {refusal:?}")
    };
    let Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { owner, .. }) =
        error.authored_retirement_refusal()
    else {
        panic!("expected OwnedPublicationRequiresCascade: {refusal:?}")
    };
    assert_eq!(owner, model.native_id, "the refusal names the source model");
    assert_eq!(document.selection(&request).unwrap(), selection);
    assert_eq!(world::model(&host, branch), model);
    assert_eq!(world::outputs(&host, branch), outputs);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

/// Settling producers on the document branch outside the history moves its
/// head: every later selection and checkout refuses `HeadDiverged` first.
#[test]
fn out_of_history_settlement_on_the_document_branch_diverges_its_head() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let Edited {
        document,
        before,
        branch,
        selection,
        outputs,
    } = edited(&host, &request, true);
    let model = world::model(&host, branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let diverged = |cause: &WorthQueryHistoryNavigationAdvanceDenial| {
        matches!(
            cause,
            WorthQueryHistoryNavigationAdvanceDenial::Navigation(
                WorthQueryHistoryNavigationDenial::Document(
                    WorthQueryDocumentHistoryDenial::HeadDiverged
                )
            )
        )
    };
    let current = document.selection(&request).expect_err("diverged head");
    assert!(diverged(&current), "{current:?}");
    let refusal = checkout(&document, &request, before, selection)
        .expect_err("a diverged head must refuse checkout");
    assert!(
        matches!(&refusal, ("prepare", cause) if diverged(cause)),
        "{refusal:?}"
    );
    assert_eq!(world::model(&host, branch), model);
    assert_eq!(world::outputs(&host, branch), outputs);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
