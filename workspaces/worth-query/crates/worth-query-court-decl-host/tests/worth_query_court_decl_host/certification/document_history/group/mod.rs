//! Public grouped edits preserve one real document and actual staged effects.
use super::recording::close;
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationHistoryDenial, WorthQueryDocumentHistoryDenial,
        WorthQueryGroupedHistoryDocument, WorthQueryHistoryGroupDenial,
        WorthQueryHistoryGroupPosture as Posture, WorthQuerySourceCompletion,
    },
    declaration::application_program::{ApplicationActionReference, ApplicationRootComposition},
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::navigation::Navigation,
    programs::{completion::RetentionCompletion, DocumentRetentionFeature},
    retention_entry::{SetRetentionBinding, SetRetentionInput, SetRetentionIntent},
};

mod cancellation;
mod guide;
mod handoff;
mod identity;
mod predecessor;
mod race;
mod recovery;
mod refusal;
mod resources;
mod retries;

type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;
type Group<'r> = WorthQueryGroupedHistoryDocument<
    'r,
    DocumentRetentionSchema,
    RetentionHistory,
    SetRetentionIntent,
    RetentionProgramP0,
    Action,
>;

fn input(days: u64) -> SetRetentionIntent {
    SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: days,
        },
    }
}

fn begin<'r>(host: &App, document: Document<'r>, second: u64, key: u128) -> Group<'r> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    guide::open_group(
        document,
        host,
        &host.request(&principal, &scope),
        key,
        second,
    )
    .unwrap_or_else(|(_, cause)| panic!("group description: {cause:?}"))
}

fn until(host: &Runtime, group: &mut Group<'_>, stop: impl Fn(&Group<'_>) -> bool) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for _ in 0..48 {
        if stop(group) {
            return;
        }
        let (document, execution) = group.parts();
        execution.advance(document, &request).unwrap();
    }
    panic!("finite group did not reach its required posture")
}

#[test]
fn two_ordinary_edits_select_one_history_edge_and_undo_redo_its_complete_endpoint() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut group = begin(&host, document, 5, 0x9182_5000);
    until(&host, &mut group, |group| group.completed_members() == 1);
    {
        let (document, execution) = group.parts();
        assert_eq!(value(&host, document), 7);
        assert_eq!(document.current_step(&request).unwrap(), root);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            3
        );
        let WorthQuerySourceCompletion::Fresh {
            receipt, result, ..
        } = execution.completed_source(0).unwrap().evidence()
        else {
            panic!("actual first publication")
        };
        assert_eq!(result.retention_days, 3);
        let staged = execution.read_staged(document, &request).unwrap();
        assert_eq!(
            staged.receipt().inspect().basis().branch(),
            receipt.commit_reference().branch_id.0
        );
        assert_eq!(
            staged.receipt().inspect().basis().version(),
            receipt.commit_reference().version_id.0
        );
    }
    for _ in 0..48 {
        if group.posture() == Posture::Selected {
            break;
        }
        guide::advance_group(&mut group, &request).unwrap();
    }
    assert_eq!(group.posture(), Posture::Selected);
    let step = {
        let (document, execution) = group.parts();
        execution.complete(document, &request).unwrap()
    };
    assert_eq!(step.parent(), Some(root.id()));
    assert_eq!(value(&host, group.document()), 5);
    {
        let (document, execution) = group.parts();
        assert_eq!(execution.complete(document, &request).unwrap(), step);
    }
    let finished = group.finish(&request).unwrap();
    assert_eq!(
        finished.kind,
        worth_query_host::facade::application_entry::WorthQueryHistoryGroupCompletionKind::Recorded
    );
    assert_eq!(finished.completions.len(), 2);
    let mut originals = finished.completions.iter();
    let (_, first) = originals.next().unwrap();
    let (_, second) = originals.next().unwrap();
    let (
        WorthQuerySourceCompletion::Fresh { receipt: first, .. },
        WorthQuerySourceCompletion::Fresh {
            receipt: second, ..
        },
    ) = (first, second)
    else {
        panic!("actual ordered publications")
    };
    assert_eq!(
        second.commit_reference().parents,
        [first.commit_reference().commit_id.clone()]
    );
    assert_eq!(
        second.commit_reference().branch_id,
        first.commit_reference().branch_id
    );
    assert_eq!(
        step.endpoint().selected_commit(),
        second.committed_product_publication().composite_commit()
    );
    for ((_, completion), expected) in finished.completions.iter().zip([3, 5]) {
        let WorthQuerySourceCompletion::Fresh { result, .. } = completion else {
            panic!("fresh original result")
        };
        assert_eq!(result.retention_days, expected);
    }
    let document = finished.document;
    let page = document
        .history_page(
            &request,
            worth_query_host::facade::application_entry::WorthQueryHistoryPageLimits {
                maximum_entries: std::num::NonZeroUsize::new(8).unwrap(),
                maximum_examined_entries: std::num::NonZeroUsize::new(8).unwrap(),
                maximum_owned_bytes: std::num::NonZeroUsize::new(16384).unwrap(),
            },
            None,
        )
        .unwrap();
    assert_page_entries(
        &page
            .rows()
            .iter()
            .map(|row| row.step().clone())
            .collect::<Vec<_>>(),
        &[root.clone(), step.clone()],
    );
    let undo = super::navigation::complete(
        &host,
        document
            .navigate::<Navigation>()
            .undo_step()
            .expected_selection(document.selection(&request).unwrap())
            .idempotency(0x9182_5010)
            .prepare(&request)
            .unwrap(),
    );
    assert_eq!(undo.requested_node(), root.id());
    assert_eq!(value(&host, &document), 7);
    let redo = super::navigation::complete(
        &host,
        document
            .navigate::<Navigation>()
            .redo_to(step.id())
            .expected_selection(document.selection(&request).unwrap())
            .idempotency(0x9182_5011)
            .prepare(&request)
            .unwrap(),
    );
    assert_eq!(redo.requested_node(), step.id());
    assert_eq!(value(&host, &document), 5);
    assert_eq!(value(&host, &untouched), 7);
    drop(finished.completions);
    close(&host, document, 4);
    close(&host, untouched, 1);
}
