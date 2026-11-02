//! A new click returns its original selector alongside the owned first advance.
use super::*;

type Navigation<'d, 'r, O> = WorthQueryHistoryNavigation<
    'd,
    'r,
    ConsumerSchema,
    History,
    PlanarHistoryNavigation<ConsumerSchema>,
    OrdinaryHistoryPayload,
    O,
>;

pub(super) fn start<'d, 'r, O>(
    document: &'d WorthQueryApplicationHistoryDocument<
        'r,
        ConsumerSchema,
        History,
        OrdinaryHistoryPayload,
        O,
    >,
    request: &Request<'_, '_, '_>,
    node: WorthQueryHistoryNodeId,
    key: u128,
) -> Result<
    (
        WorthQueryHistorySelection,
        WorthQueryHistoryNavigationAdvance<Navigation<'d, 'r, O>>,
    ),
    WorthQueryHistoryNavigationAdvanceDenial,
>
where
    O: WorthQueryHistoryOutputsMode<ConsumerSchema, History>,
{
    // compiled-example: begin
    let expected = document.selection(request)?;
    let navigation = document
        .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
        .checkout(node)
        .expected_selection(expected.clone())
        .idempotency(key)
        .prepare(request)?;
    Ok((expected, navigation.advance(request)))
    // compiled-example: end
}
