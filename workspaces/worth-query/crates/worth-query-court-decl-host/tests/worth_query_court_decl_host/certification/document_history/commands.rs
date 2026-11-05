//! One description adapter admits real commands for an independent scalar schema.
use super::recording::{begin, close, finish, ready};
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::declaration::application_history::ApplicationHistoryRecentSteps;
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::{
    navigation::Navigation,
    retention_policy::{Bookmark, Window},
};

fn limits() -> WorthQueryHistoryPageLimits {
    WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(8).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(8).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}

#[test]
fn description_format_binding_and_session_refuse_before_commands_with_a_lawful_counterpart() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let sibling = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let foreign_document = attach(&foreign, head(&foreign), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let foreign_scope = request_scope();
    let foreign_principal = authenticate_operator(foreign.installed_schema(), &foreign_scope);
    let before = document.selection(&request).unwrap();
    let page = document.history_page(&request, limits(), None).unwrap();
    let context = page.context().clone();
    assert_eq!(
        context.version(),
        WorthQueryHistoryDescriptionContext::CURRENT_VERSION
    );
    assert_eq!(
        context.schema_binding(),
        &host.installed_schema().binding_identity()
    );
    assert_eq!(
        context,
        *document
            .redo_choices(&request, limits(), None)
            .unwrap()
            .children()
            .context()
    );
    let sibling_context = sibling
        .history_page(&request, limits(), None)
        .unwrap()
        .context()
        .clone();
    assert_ne!(context.document(), sibling_context.document());
    let foreign_context = foreign_document
        .history_page(
            &foreign.request(&foreign_principal, &foreign_scope),
            limits(),
            None,
        )
        .unwrap()
        .context()
        .clone();
    for (invalid, expected) in [
        (
            context.clone().for_version(99),
            WorthQueryHistoryCommandContextDenial::UnsupportedVersion {
                received: 99,
                supported: 1,
            },
        ),
        (
            sibling_context,
            WorthQueryHistoryCommandContextDenial::ForeignDocument,
        ),
        (
            foreign_context,
            WorthQueryHistoryCommandContextDenial::ForeignSchemaBinding,
        ),
    ] {
        assert_eq!(document.commands(&invalid).err().unwrap(), expected);
        assert_eq!(document.selection(&request).unwrap(), before);
        assert_eq!(value(&host, &document), 7);
        assert!(host.branches().pending_cleanup().is_empty());
    }
    let root = document.current_step(&request).unwrap();
    let preview = document
        .commands(&context)
        .unwrap()
        .preview(&request, root.id())
        .unwrap();
    assert_eq!(
        preview.read(&document, &request).unwrap().rows()[0].retention_days,
        7
    );
    preview.close();
    drop(page);
    close(&host, document, 1);
    close(&host, sibling, 1);
    close(&foreign, foreign_document, 1);
}

#[test]
fn the_same_command_adapter_keeps_original_navigation_policy_and_preview_custody() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut first = None;
    for (index, days) in [3, 5, 6].into_iter().enumerate() {
        let (next, step) = finish(
            &host,
            ready(
                &host,
                begin(
                    &host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_7100 + index as u64,
                ),
            ),
            days,
        );
        document = next;
        first.get_or_insert(step);
    }
    let first = first.unwrap();
    let context = document
        .history_page(&request, limits(), None)
        .unwrap()
        .context()
        .clone();
    let preview = document
        .commands(&context)
        .unwrap()
        .preview(&request, first.id())
        .unwrap();
    {
        let mut commands = document.commands(&context).unwrap();
        commands
            .bookmark::<Bookmark>(&request, root.id(), true)
            .unwrap();
        let status = commands
            .window(ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN))
            .advance::<Window>(&request)
            .unwrap();
        assert!(status.pending_window.is_none());
    }
    assert!(matches!(
        document
            .commands(&context)
            .unwrap()
            .preview(&request, first.id()),
        Err(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::HistoryNodeExcluded
        ))
    ));
    assert_eq!(
        preview.read(&document, &request).unwrap().rows()[0].retention_days,
        3
    );
    let expected = document.selection(&request).unwrap();
    let mut selected = None;
    // Only the transient frontend is dropped between requests. The same actual
    // Native/SDK slots retain the original attempt, expected selection and key.
    for _ in 0..32 {
        let commands = document.commands(&context).unwrap();
        let owner = commands
            .navigate::<Navigation>()
            .checkout(root.id())
            .expected_selection(expected.clone())
            .idempotency(0x9182_7110)
            .prepare(&request)
            .unwrap();
        match owner.advance(&request) {
            WorthQueryHistoryNavigationAdvance::Selected(done) => {
                selected = Some(done);
                break;
            }
            WorthQueryHistoryNavigationAdvance::Pending(owner) => drop(owner),
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("original receiver refused: {cause:?}")
            }
        }
    }
    assert_eq!(selected.unwrap().selection().step().id(), root.id());
    assert_eq!(value(&host, &document), 7);
    preview.close();
    document
        .commands(&context)
        .unwrap()
        .bookmark::<Bookmark>(&request, root.id(), false)
        .unwrap();
    for _ in 0..16 {
        if !document
            .history_retention_status(&request)
            .unwrap()
            .maintenance_pending()
        {
            break;
        }
        document
            .commands(&context)
            .unwrap()
            .maintain(&request)
            .unwrap();
    }
    close(&host, document, 5);
}
