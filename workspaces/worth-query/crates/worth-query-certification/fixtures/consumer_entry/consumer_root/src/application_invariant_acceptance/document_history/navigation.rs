//! A queued gesture preserves original descriptive intent, expectation and key.
use super::*;

pub(super) struct Command {
    pub context: WorthQueryHistoryDescriptionContext,
    pub intent: WorthQueryHistoryNavigationIntent,
    pub expected: WorthQueryHistorySelection,
    pub key: u128,
}

pub(super) enum Failure {
    Context(WorthQueryHistoryCommandContextDenial),
    Preparation(WorthQueryHistoryNavigationAdvanceDenial),
    Advancement {
        cause: WorthQueryHistoryNavigationAdvanceDenial,
        posture: WorthQueryHistoryNavigationPosture,
    },
}

impl std::fmt::Debug for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Context(cause) => formatter.debug_tuple("Context").field(cause).finish(),
            Self::Preparation(cause) => formatter.debug_tuple("Preparation").field(cause).finish(),
            Self::Advancement { cause, posture } => formatter
                .debug_struct("Advancement")
                .field("cause", cause)
                .field("posture", posture)
                .finish(),
        }
    }
}

pub(super) enum Progress {
    Pending(WorthQueryHistoryNavigationPosture),
    Selected(WorthQueryHistoryNavigationCompletion),
}

pub(super) fn capture(
    document: &Configured<'_>,
    request: &Request<'_, '_, '_>,
    context: &WorthQueryHistoryDescriptionContext,
    intent: WorthQueryHistoryNavigationIntent,
    key: u128,
) -> Result<Command, Failure> {
    document
        .check_history_command_context(context)
        .map_err(Failure::Context)?;
    let expected = document.selection(request).map_err(Failure::Preparation)?;
    Ok(Command {
        context: context.clone(),
        intent,
        expected,
        key,
    })
}

pub(super) fn advance(
    document: &mut Configured<'_>,
    request: &Request<'_, '_, '_>,
    command: &Command,
) -> Result<Progress, Failure> {
    // compiled-example: begin
    let commands = document
        .commands(&command.context)
        .map_err(Failure::Context)?;
    let navigation = commands.navigate::<PlanarHistoryNavigation<ConsumerSchema>>();
    let target = match command.intent {
        WorthQueryHistoryNavigationIntent::Checkout(node) => navigation.checkout(node),
        WorthQueryHistoryNavigationIntent::UndoStep => navigation.undo_step(),
        WorthQueryHistoryNavigationIntent::RedoTo(child) => navigation.redo_to(child),
    };
    let owner = target
        .expected_selection(command.expected.clone())
        .idempotency(command.key)
        .prepare(request)
        .map_err(Failure::Preparation)?;
    match owner.advance(request) {
        WorthQueryHistoryNavigationAdvance::Selected(done) => Ok(Progress::Selected(done)),
        WorthQueryHistoryNavigationAdvance::Pending(owner) => {
            Ok(Progress::Pending(owner.posture()))
        }
        WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => Err(Failure::Advancement {
            cause,
            posture: owner.posture(),
        }),
    }
    // compiled-example: end
}

pub(super) fn complete(
    document: &mut Configured<'_>,
    app: &Runtime,
    command: &Command,
) -> WorthQueryHistoryNavigationCompletion {
    for _ in 0..96 {
        match model::with_request(app, |request| advance(document, request, command))
            .unwrap_or_else(|cause| panic!("original document command: {cause:?}"))
        {
            Progress::Pending(posture) => {
                assert_ne!(posture, WorthQueryHistoryNavigationPosture::Selected)
            }
            Progress::Selected(done) => return done,
        }
    }
    panic!("finite native/output navigation did not settle");
}
