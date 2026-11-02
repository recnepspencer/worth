//! A renderer may discard these bounded descriptions without touching source owners.
use super::*;

pub(super) struct View {
    pub context: WorthQueryHistoryDescriptionContext,
    pub rows: Vec<WorthQueryHistoryPageEntry>,
    pub current: WorthQueryHistoryNodeId,
    pub after: Option<WorthQueryHistoryPageCursor>,
}

pub(super) fn limits() -> WorthQueryHistoryPageLimits {
    WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(8).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(8).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}

pub(super) fn page(
    document: &Configured<'_>,
    request: &Request<'_, '_, '_>,
    after: Option<&WorthQueryHistoryPageCursor>,
) -> Result<View, WorthQueryApplicationHistoryDenial> {
    let page = document.history_page(request, limits(), after)?;
    Ok(View {
        context: page.context().clone(),
        current: page.current_step_id(),
        after: page.continuation(),
        rows: page.rows().iter().map(|row| row.step().clone()).collect(),
    })
}
