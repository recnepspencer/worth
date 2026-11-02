//! The exact guide bytes run against real window, bookmark and disposal owners.
use super::*;
use worth_query_host::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    application_entry::{
        WorthQueryApplicationHistoryCloseFailure, WorthQueryApplicationRequest,
        WorthQueryDocumentHistoryCloseReceipt, WorthQueryHistoryRetentionStatus,
    },
};
type Request<'a, 'b, 'c> = WorthQueryApplicationRequest<'a, 'b, 'c, DocumentRetentionSchema>;

// compiled-example: begin
fn protect_and_shrink(
    document: &mut Document<'_>,
    request: &Request<'_, '_, '_>,
    bookmark: WorthQueryHistoryNodeId,
    target: ApplicationHistoryRecentSteps,
) -> Result<WorthQueryHistoryRetentionStatus, Failure> {
    document.maintain_history(request)?;
    document.set_history_bookmark::<Bookmark>(request, bookmark, true)?;
    document
        .set_history_window(target)
        .advance::<Window>(request)
}

fn dispose_document<'runtime>(
    document: Document<'runtime>,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> Result<
    WorthQueryDocumentHistoryCloseReceipt,
    WorthQueryApplicationHistoryCloseFailure<'runtime, DocumentRetentionSchema, RetentionHistory>,
> {
    document.close(phase)
}
// compiled-example: end

#[test]
fn the_guide_preserves_a_partial_window_and_closes_the_same_document() {
    let host = publish_on_first_program();
    let (mut document, root) = super::cancellation::filled(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let target = ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN);
    let first = protect_and_shrink(&mut document, &request, root.id(), target).unwrap();
    assert_eq!((first.bookmarks, first.pending_window), (1, Some(target)));
    window(&host, &mut document, 1);
    assert_eq!(
        document
            .history_retention_status(&request)
            .unwrap()
            .bookmarks,
        1
    );
    document
        .set_history_bookmark::<Bookmark>(&request, root.id(), false)
        .unwrap();
    for _ in 0..8 {
        if !document
            .history_retention_status(&request)
            .unwrap()
            .maintenance_pending()
        {
            break;
        }
        document.maintain_history(&request).unwrap();
    }
    let receipt = host
        .with_host_advancement(|phase| dispose_document(document, &phase).unwrap())
        .unwrap();
    assert_eq!(receipt.closed_heads(), 5);
    assert!(host.branches().pending_cleanup().is_empty());
}
