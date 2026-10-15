//! An ordinary notification attempt retains native custody on every refusal.
use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{BankSchema, NotifyEstateDeath},
};
use bank_server::{
    BankApplication, BankAuthenticatedPrincipal, BankCommitReceipt, BankIdentityRuntime,
    BankMutationControls, DeathNotificationAction,
};
use worth_query_host::facade::{
    application_entry::*, declaration::application_program::ApplicationSourceCompletion,
};
type Operation = WorthQuerySourceOnlyOperation<
    BankSchema,
    NotifyEstateDeath,
    BankApplication,
    DeathNotificationAction,
    ApplicationSourceCompletion,
>;
type Completed = WorthQueryCompletedSourceOperation<
    BankSchema,
    NotifyEstateDeath,
    BankApplication,
    DeathNotificationAction,
    ApplicationSourceCompletion,
>;
type Terminal = WorthQueryTerminalSourceOperation<
    BankSchema,
    NotifyEstateDeath,
    BankApplication,
    DeathNotificationAction,
    ApplicationSourceCompletion,
>;
type Advance = WorthQuerySourceOnlyAdvance<Operation, Completed, Terminal>;

pub(super) enum NotificationCustody {
    WrongInput(EstateAction),
    Construction {
        description: WorthQueryOperationDescription<BankSchema, NotifyEstateDeath>,
        denial: WorthQueryOperationConstructionDenial,
    },
    Native(Advance),
    Recovered(Completed),
    CloseRefused {
        completed: Completed,
        denial: WorthQueryOperationCloseDenial,
    },
}
pub(super) enum ExampleFailure {
    Message(String),
    Notification(Box<NotificationCustody>),
    Host(Box<bank_http_adapter::BankRailCompletionCloseFailure>),
    SourceAndHost {
        original: Box<ExampleFailure>,
        close: Result<
            Box<bank_http_adapter::BankRailCompletionClose>,
            Box<bank_http_adapter::BankRailCompletionCloseFailure>,
        >,
    },
}
impl From<String> for ExampleFailure {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}
impl std::fmt::Debug for ExampleFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
            Self::Host(failure) => write!(f, "callback host close refused: {failure}"),
            Self::SourceAndHost { original, close } => match close {
                Ok(close) => write!(
                    f,
                    "{original:?}; callback host assessment: {:?}",
                    close.assessment()
                ),
                Err(failure) => {
                    write!(
                        f,
                        "{original:?}; callback host close also refused: {failure}"
                    )
                }
            },
            Self::Notification(custody) => {
                let label = match custody.as_ref() {
                    NotificationCustody::WrongInput(action) => {
                        let _ = action;
                        "wrong original input"
                    }
                    NotificationCustody::Construction {
                        description,
                        denial,
                    } => {
                        let _ = (description, denial);
                        "construction refusal"
                    }
                    NotificationCustody::Native(advance) => {
                        let _ = advance;
                        "native source custody"
                    }
                    NotificationCustody::Recovered(completed) => {
                        let _ = completed;
                        "recovered completion requires lawful close"
                    }
                    NotificationCustody::CloseRefused { completed, denial } => {
                        let _ = (completed, denial);
                        "cleanup refusal"
                    }
                };
                write!(f, "notification retains {label}")
            }
        }
    }
}

pub(super) fn issue(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    action: EstateAction,
) -> Result<BankCommitReceipt, ExampleFailure> {
    let fail = |custody| ExampleFailure::Notification(Box::new(custody));
    let input = NotifyEstateDeath::try_from(action)
        .map_err(|action| fail(NotificationCustody::WrongInput(action)))?;
    let scope = super::support::request_scope();
    let operation = runtime
        .mutate(bank_server::mutations::notify_estate_death(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(
            scope.clone(),
            BankIdempotencyKey::new("public-inbound-example").unwrap(),
        ))
        .retain_operation()
        .map_err(|(description, denial)| {
            fail(NotificationCustody::Construction {
                description,
                denial,
            })
        })?;
    let advance = operation.advance(&runtime.request(principal, &scope));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = advance else {
        return Err(fail(NotificationCustody::Native(advance)));
    };
    // This journey claims a new physical outbox, so recovered evidence cannot
    // stand in for its original handler result. Preserve that owner explicitly.
    if !matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ) {
        return Err(fail(NotificationCustody::Recovered(completed)));
    }
    let closed = completed
        .close_with_evidence(&runtime.request(principal, &scope))
        .map_err(|(completed, denial)| {
            fail(NotificationCustody::CloseRefused { completed, denial })
        })?;
    Ok(BankCommitReceipt::from_closed_source(closed))
}
