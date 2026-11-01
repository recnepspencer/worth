//! A button is descriptive input; retries preserve its original expected selection and key.
use super::*;

#[derive(Clone, Copy)]
pub(super) enum Button {
    Undo,
    RedoTo(WorthQueryHistoryNodeId),
}

type Navigation<'d, 'r, O> = WorthQueryHistoryNavigation<
    'd,
    'r,
    ConsumerSchema,
    History,
    PlanarHistoryNavigation<ConsumerSchema>,
    OrdinaryHistoryPayload,
    O,
>;
type Started<'d, 'r, O> = (
    Button,
    WorthQueryHistorySelection,
    WorthQueryHistoryNavigationAdvance<Navigation<'d, 'r, O>>,
);

pub(super) fn start<'d, 'r, O>(
    document: &'d WorthQueryApplicationHistoryDocument<
        'r,
        ConsumerSchema,
        History,
        OrdinaryHistoryPayload,
        O,
    >,
    request: &Request<'_, '_, '_>,
    button: Button,
    key: u128,
) -> Result<Started<'d, 'r, O>, WorthQueryHistoryNavigationAdvanceDenial>
where
    O: WorthQueryHistoryOutputsMode<ConsumerSchema, History>,
{
    let expected = document.selection(request)?;
    prepare(document, request, button, expected, key)
}

pub(super) fn prepare<'d, 'r, O>(
    document: &'d WorthQueryApplicationHistoryDocument<
        'r,
        ConsumerSchema,
        History,
        OrdinaryHistoryPayload,
        O,
    >,
    request: &Request<'_, '_, '_>,
    button: Button,
    expected: WorthQueryHistorySelection,
    key: u128,
) -> Result<Started<'d, 'r, O>, WorthQueryHistoryNavigationAdvanceDenial>
where
    O: WorthQueryHistoryOutputsMode<ConsumerSchema, History>,
{
    // compiled-example: begin
    let builder = document.navigate::<PlanarHistoryNavigation<ConsumerSchema>>();
    let target = match button {
        Button::Undo => builder.undo_step(),
        Button::RedoTo(child) => builder.redo_to(child),
    };
    let navigation = target
        .expected_selection(expected.clone())
        .idempotency(key)
        .prepare(request)?;
    Ok((button, expected, navigation.advance(request)))
    // compiled-example: end
}
