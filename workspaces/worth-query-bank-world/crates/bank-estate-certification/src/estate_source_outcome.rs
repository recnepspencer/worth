//! Certification observations preserve the actual native source custody.
use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{BankSchema, EstateMutationResult},
};
use bank_server::{BankApplication, BankCommitReceipt};
use worth_query_decl::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
    application_program::ApplicationSourceCompletion,
};
use worth_query_host::facade::application_entry::*;

/// The exact native owning operation; this certification type adds no engine.
pub type EstateSourceOperation<I, A> =
    WorthQuerySourceOnlyOperation<BankSchema, I, BankApplication, A, ApplicationSourceCompletion>;
/// One real native source phase, with distinct completed and terminal custody.
pub type EstateSourceAdvance<I, A> = WorthQuerySourceOnlyAdvance<
    EstateSourceOperation<I, A>,
    WorthQueryCompletedSourceOperation<
        BankSchema,
        I,
        BankApplication,
        A,
        ApplicationSourceCompletion,
    >,
    WorthQueryTerminalSourceOperation<
        BankSchema,
        I,
        BankApplication,
        A,
        ApplicationSourceCompletion,
    >,
>;

/// One certification attempt. Every incomplete outcome retains its original native owner.
pub enum EstateSourceCommandOutcome<I: ApplicationMutationIntent<BankSchema>, A> {
    WrongInput {
        action: EstateAction,
        key: BankIdempotencyKey,
    },
    Construction(bank_server::BankOperationConstructionRefusal<I>),
    Native(EstateSourceAdvance<I, A>),
    CloseRefused {
        completed: WorthQueryCompletedSourceOperation<
            BankSchema,
            I,
            BankApplication,
            A,
            ApplicationSourceCompletion,
        >,
        denial: WorthQueryOperationCloseDenial,
    },
    Fresh {
        receipt: BankCommitReceipt,
        result: EstateMutationResult,
    },
    Recovered {
        receipt: BankCommitReceipt,
    },
}

// One close attempt only. Native progress/refusal/terminal is returned unchanged;
// no consumer loop drives recovery or repeats the original source command.
pub(super) fn close_completed<I, A>(
    advance: EstateSourceAdvance<I, A>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
) -> EstateSourceCommandOutcome<I, A>
where
    I: ApplicationMutationIntent<BankSchema>,
    I::Binding: ApplicationMutationBinding<BankSchema, Result = EstateMutationResult>,
{
    let WorthQuerySourceOnlyAdvance::Completed(completed) = advance else {
        return EstateSourceCommandOutcome::Native(advance);
    };
    let fresh = match completed.evidence() {
        WorthQuerySourceCompletion::Fresh { result, .. } => Some(*result),
        WorthQuerySourceCompletion::Recovered { .. } => None,
    };
    match completed.close_with_evidence(request) {
        Ok(closed) => {
            let receipt = BankCommitReceipt::from_closed_source(closed);
            match fresh {
                Some(result) => EstateSourceCommandOutcome::Fresh { receipt, result },
                None => EstateSourceCommandOutcome::Recovered { receipt },
            }
        }
        Err((completed, denial)) => EstateSourceCommandOutcome::CloseRefused { completed, denial },
    }
}
