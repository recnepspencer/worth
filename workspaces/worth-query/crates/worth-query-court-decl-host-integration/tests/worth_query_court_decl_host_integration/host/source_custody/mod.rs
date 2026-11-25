//! Interactive source custody and bundled bounds over a real Directory Home
//! opened under bounded serial limits and a local-process principal.
mod bounded_history_reopen;
mod commit_and_decline;
mod interrupted_frames;

use super::application_startup::{close_home, local_principal, open_home, read_balance, scope};
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::{
        WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
        WorthQueryOperationDescription, WorthQueryProjectedQuery, WorthQueryProjectedValue,
        WorthQuerySourceCompletion, WorthQuerySourcelessMutation, WorthQueryTerminalSourceEvidence,
    },
    application_installation::WorthQueryApplicationLimits,
    declaration::application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
    domain::ApplicationSchema,
    WorthQuerySourceCustody, WorthQuerySourceCustodyPending, WorthQuerySourceCustodyStep,
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};

type Schema = super::application_startup::Schema;
type Balance = fixture::native_mutation::SetAccountBalance;
type Custody =
    WorthQuerySourceCustody<Schema, Balance, fixture::HomeProgram, fixture::BalanceAction>;
type Step =
    WorthQuerySourceCustodyStep<Schema, Balance, fixture::HomeProgram, fixture::BalanceAction>;
type Key<S, I> =
    <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::IdempotencyKey;

/// A generic application helper: one bundled bound captures any sourceless command.
fn capture<S, I>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    intent: I,
    key: Key<S, I>,
) -> WorthQueryOperationDescription<S, I>
where
    S: ApplicationSchema,
    I: WorthQuerySourcelessMutation<S>,
{
    match request
        .mutate(intent)
        .without_source()
        .idempotency_owned(key)
    {
        Ok(described) => described,
        Err((_, _, denial)) => panic!("the command's input is captured: {denial:?}"),
    }
}

/// A generic application helper: one bundled bound executes any projected query.
fn read<S, Q>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    intent: Q,
) -> Option<Vec<WorthQueryProjectedValue<S, Q>>>
where
    S: ApplicationSchema,
    Q: WorthQueryProjectedQuery<S>,
    WorthQueryProjectedValue<S, Q>: Clone,
{
    request
        .query(intent)
        .execute()
        .ok()
        .map(|result| result.rows().to_vec())
}

fn set_balance(balance: u64) -> Balance {
    Balance {
        account: "account-001".to_owned(),
        balance,
    }
}

fn account() -> fixture::balance_query::AccountRead {
    fixture::balance_query::AccountRead("account-001".to_owned())
}

/// The settled outcome of a step that must not still be pending.
fn settled(step: Step) -> String {
    match step {
        Step::Committed {
            evidence: WorthQuerySourceCompletion::Fresh { result, .. },
            ..
        } => format!("committed {result}"),
        Step::Committed { .. } => "committed without a live result".to_owned(),
        Step::Declined {
            evidence: WorthQueryTerminalSourceEvidence::Outcome(outcome),
            ..
        } => format!("declined {outcome:?}"),
        Step::Declined { .. } => "declined without a native outcome".to_owned(),
        Step::Pending { cause, .. } => panic!("custody is still pending: {cause:?}"),
    }
}

/// The retained custody and native cause of a step that must be pending.
fn pending(step: Step) -> (Custody, WorthQuerySourceCustodyPending) {
    match step {
        Step::Pending { custody, cause } => (custody, cause),
        settled => panic!("custody settled early: {}", self::settled(settled)),
    }
}
