use bank_domain::schema::{EstateMutationDenial, EstateMutationResult};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::primary_graph::{
    HandlerExecutionDenial, MutationHandlerExecutionDenial,
};

use super::BankEstateProgressionDenial;
use crate::{BankCommitReceipt, BankMutationCommitOutcome};

pub(super) fn program_outcome(
    outcome: Result<
        WorthQueryApplicationMutationOutcome<EstateMutationDenial, EstateMutationResult>,
        WorthQueryApplicationRequestMutationDenial,
    >,
    operation: &'static str,
    map_handler: impl FnOnce(
        HandlerExecutionDenial,
    ) -> Result<BankEstateProgressionDenial, HandlerExecutionDenial>,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Handler(denial),
        )) => match map_handler(denial) {
            Ok(denial) => return Err(denial),
            Err(denial) => {
                return Err(BankEstateProgressionDenial::ApplicationEntry(
                    WorthQueryApplicationRequestMutationDenial::Handler(
                        MutationHandlerExecutionDenial::Handler(denial),
                    ),
                ));
            }
        },
        Err(denial) => return Err(BankEstateProgressionDenial::ApplicationEntry(denial)),
    };
    Ok(match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => {
            BankMutationCommitOutcome::Committed(commit_receipt(receipt))
        }
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => {
            BankMutationCommitOutcome::AlreadyCommitted(commit_receipt(receipt))
        }
        WorthQueryApplicationMutationOutcome::Commit(outcome) => outcome.into(),
        WorthQueryApplicationMutationOutcome::Cancelled => BankMutationCommitOutcome::Cancelled,
        WorthQueryApplicationMutationOutcome::DeadlineExceeded => {
            BankMutationCommitOutcome::TimedOut
        }
        WorthQueryApplicationMutationOutcome::DomainDenied(EstateMutationDenial::InputVariant) => {
            return Err(BankEstateProgressionDenial::CommandInput(operation))
        }
        WorthQueryApplicationMutationOutcome::DomainDenied(EstateMutationDenial::Proposal(
            denial,
        )) => return Err(BankEstateProgressionDenial::Proposal(denial)),
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift => {
            return Err(BankEstateProgressionDenial::IdempotencyIntentDrift);
        }
    })
}

fn commit_receipt(
    receipt: worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
) -> BankCommitReceipt {
    crate::operation_commit::commit_receipt(receipt)
}
