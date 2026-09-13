use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption;

use super::{mutations, BankMutationContract, BankReadyMutation};
use crate::ordinary::mutation::{
    BankMutationControls, BankMutationDenial, BankMutationOutcome, BankMutationStatus,
};
use crate::{
    BankAuthenticatedPrincipal, BankAuthorizedProposal, BankCommitPreparationDenial,
    BankIdentityRuntime, BankMutationCommitOutcome, BankMutationProjectionWork,
    BankOperationAdmissionError, BankOperationProposalError, BankOperationProposals,
    BankSendMoneyPreparation,
};
use bank_domain::schema::BankSchema;
use worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions;

mod workflow;

trait ExecutableBankMutation: BankMutationContract {
    fn execute(
        self,
        runtime: &BankIdentityRuntime,
        principal: &BankAuthenticatedPrincipal,
        preconditions: TypedMutationPreconditions<BankSchema, Self::Operation, Self::Scope>,
        controls: &BankMutationControls,
    ) -> BankMutationOutcome;
}

macro_rules! expose_execute {
    ($($mutation:ty => ($operation:ty, $scope:ty)),+ $(,)?) => {$(
        impl BankReadyMutation<'_, '_, $mutation, $operation, $scope> {
            pub fn execute(self) -> BankMutationOutcome {
                self.mutation.execute(
                    self.runtime,
                    self.principal,
                    self.preconditions,
                    &self.controls,
                )
            }
        }
    )+};
}

expose_execute!(
    mutations::CreatePersonalAccountMutation => (
        bank_domain::schema::CreatePersonalAccountOperation,
        bank_domain::schema::Institution
    ),
    mutations::CreateBusinessAccountMutation => (
        bank_domain::schema::CreateBusinessAccountOperation,
        bank_domain::schema::Institution
    ),
    mutations::OpeningFundingMutation => (
        bank_domain::schema::ApplyOpeningFundingOperation,
        bank_domain::schema::Institution
    ),
    mutations::DepositMutation => (
        bank_domain::schema::DepositOperation,
        bank_domain::schema::Institution
    ),
    mutations::WithdrawalMutation => (
        bank_domain::schema::WithdrawOperation,
        bank_domain::schema::Institution
    ),
    mutations::SendMoneyMutation => (
        bank_domain::schema::SendMoneyOperation,
        bank_domain::schema::Account
    ),
    mutations::ApprovePaymentMutation => (
        bank_domain::schema::ApprovePaymentOperation,
        bank_domain::schema::PaymentIntent
    ),
    mutations::RejectPaymentMutation => (
        bank_domain::schema::RejectPaymentOperation,
        bank_domain::schema::PaymentIntent
    ),
    mutations::GrantAccountAccessMutation => (
        bank_domain::schema::GrantAccountAuthorizationOperation,
        bank_domain::schema::Account
    ),
    mutations::RevokeAccountAccessMutation => (
        bank_domain::schema::RevokeAccountAuthorizationOperation,
        bank_domain::schema::Account
    ),
    mutations::ReverseJournalMutation => (
        bank_domain::schema::ReverseJournalOperation,
        bank_domain::schema::Institution
    ),
);

trait PreparedMutation {
    fn projection_work(&self) -> BankMutationProjectionWork;
}

impl<Operation, Input, Scope, ScopeIdentity> PreparedMutation
    for BankAuthorizedProposal<Operation, Input, Scope, ScopeIdentity>
{
    fn projection_work(&self) -> BankMutationProjectionWork {
        self.projection_work()
    }
}

fn execute_standard<Admission, Proposal>(
    controls: &BankMutationControls,
    authorize: impl FnOnce() -> Result<Admission, BankOperationAdmissionError>,
    prepare: impl FnOnce(
        Admission,
        &bank_domain::proposals::BankIdempotencyKey,
    ) -> Result<Proposal, BankOperationProposalError>,
    commit: impl FnOnce(Proposal) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial>,
) -> BankMutationOutcome
where
    Proposal: PreparedMutation,
{
    if let Some(outcome) = interrupted(controls) {
        return outcome;
    }
    let admission = match authorize() {
        Ok(admission) => admission,
        Err(denial) => return denied(map_admission_denial(denial), None),
    };
    let proposal = match prepare(admission, controls.idempotency_key()) {
        Ok(proposal) => proposal,
        Err(denial) => return denied(BankMutationDenial::from_proposal(denial), None),
    };
    let work = proposal.projection_work();
    match commit(proposal) {
        Ok(outcome) => committed(outcome, Some(work)),
        Err(denial) => denied(BankMutationDenial::Preparation(denial), Some(work)),
    }
}

macro_rules! standard_mutation {
    ($Mutation:ty, $authorize:ident, $scope:expr, $prepare:ident, $commit:ident) => {
        impl ExecutableBankMutation for $Mutation {
            fn execute(
                self,
                runtime: &BankIdentityRuntime,
                principal: &BankAuthenticatedPrincipal,
                preconditions: TypedMutationPreconditions<BankSchema, Self::Operation, Self::Scope>,
                controls: &BankMutationControls,
            ) -> BankMutationOutcome {
                execute_standard(
                    controls,
                    || {
                        runtime.$authorize(
                            principal,
                            $scope(&self.input),
                            preconditions,
                            controls.request(),
                        )
                    },
                    |admission, key| {
                        BankOperationProposals::$prepare(runtime, admission, key, &self.input)
                    },
                    |proposal| runtime.$commit(proposal),
                )
            }
        }
    };
}

standard_mutation!(
    mutations::CreatePersonalAccountMutation,
    authorize_create_personal_account,
    |input: &bank_domain::schema::CreatePersonalAccount| input.institution,
    prepare_create_personal_account,
    commit_create_personal_account
);
standard_mutation!(
    mutations::CreateBusinessAccountMutation,
    authorize_create_business_account,
    |input: &bank_domain::schema::CreateBusinessAccount| input.institution,
    prepare_create_business_account,
    commit_create_business_account
);
standard_mutation!(
    mutations::OpeningFundingMutation,
    authorize_opening_funding,
    |input: &bank_domain::schema::ApplyOpeningFunding| input.institution,
    prepare_opening_funding,
    commit_opening_funding
);
standard_mutation!(
    mutations::DepositMutation,
    authorize_deposit,
    |input: &bank_domain::schema::Deposit| input.institution,
    prepare_deposit,
    commit_deposit
);
standard_mutation!(
    mutations::WithdrawalMutation,
    authorize_withdrawal,
    |input: &bank_domain::schema::Withdraw| input.institution,
    prepare_withdrawal,
    commit_withdrawal
);
standard_mutation!(
    mutations::ApprovePaymentMutation,
    authorize_approve_payment,
    |input: &bank_domain::schema::ApprovePayment| input.payment,
    prepare_approve_payment,
    commit_approve_payment
);
standard_mutation!(
    mutations::RejectPaymentMutation,
    authorize_reject_payment,
    |input: &bank_domain::schema::RejectPayment| input.payment,
    prepare_reject_payment,
    commit_reject_payment
);
standard_mutation!(
    mutations::GrantAccountAccessMutation,
    authorize_grant_account_access,
    |input: &bank_domain::schema::GrantAccountAuthorization| input.account,
    prepare_grant_account_access,
    commit_grant_account_access
);
standard_mutation!(
    mutations::RevokeAccountAccessMutation,
    authorize_revoke_account_access,
    |input: &bank_domain::schema::RevokeAccountAuthorization| input.account,
    prepare_revoke_account_access,
    commit_revoke_account_access
);
standard_mutation!(
    mutations::ReverseJournalMutation,
    authorize_reverse_journal,
    |input: &bank_domain::schema::ReverseJournal| input.institution,
    prepare_reverse_journal,
    commit_reverse_journal
);

impl ExecutableBankMutation for mutations::SendMoneyMutation {
    fn execute(
        self,
        runtime: &BankIdentityRuntime,
        principal: &BankAuthenticatedPrincipal,
        preconditions: TypedMutationPreconditions<BankSchema, Self::Operation, Self::Scope>,
        controls: &BankMutationControls,
    ) -> BankMutationOutcome {
        if let Some(outcome) = interrupted(controls) {
            return outcome;
        }
        let admission = match runtime.authorize_send_money(
            principal,
            self.input.from,
            preconditions,
            controls.request(),
        ) {
            Ok(admission) => admission,
            Err(denial) => return denied(map_admission_denial(denial), None),
        };
        match BankOperationProposals::prepare_send_money(
            runtime,
            admission,
            controls.idempotency_key(),
            &self.input,
        ) {
            Ok(BankSendMoneyPreparation::Proposal(proposal)) => {
                let work = proposal.projection_work();
                match runtime.commit_send_money(proposal) {
                    Ok(outcome) => committed(outcome, Some(work)),
                    Err(denial) => denied(BankMutationDenial::Preparation(denial), Some(work)),
                }
            }
            Ok(BankSendMoneyPreparation::AlreadyCommitted {
                receipt,
                projection_work,
            }) => BankMutationOutcome::new(
                BankMutationStatus::AlreadyCommitted(receipt),
                Some(projection_work),
            ),
            Ok(BankSendMoneyPreparation::IntentDrift { projection_work }) => denied(
                BankMutationDenial::IdempotencyIntentDrift,
                Some(projection_work),
            ),
            Err(denial) => denied(BankMutationDenial::from_proposal(denial), None),
        }
    }
}

fn interrupted(controls: &BankMutationControls) -> Option<BankMutationOutcome> {
    controls
        .request()
        .interruption()
        .map(|interruption| match interruption {
            WorthQueryRequestInterruption::Cancelled => {
                BankMutationOutcome::new(BankMutationStatus::Cancelled, None)
            }
            WorthQueryRequestInterruption::DeadlineExceeded => {
                BankMutationOutcome::new(BankMutationStatus::DeadlineExceeded, None)
            }
        })
}

fn committed(
    outcome: BankMutationCommitOutcome,
    work: Option<BankMutationProjectionWork>,
) -> BankMutationOutcome {
    let status = match outcome {
        BankMutationCommitOutcome::ProductStale(stale) => BankMutationStatus::ProductStale(stale),
        BankMutationCommitOutcome::ProductUnpublished(unpublished) => {
            BankMutationStatus::ProductUnpublished(unpublished)
        }
        BankMutationCommitOutcome::NoEffect(no_effect) => BankMutationStatus::NoEffect(no_effect),
        BankMutationCommitOutcome::Committed(receipt) => BankMutationStatus::Committed(receipt),
        BankMutationCommitOutcome::AlreadyCommitted(receipt) => {
            BankMutationStatus::AlreadyCommitted(receipt)
        }
        BankMutationCommitOutcome::Stale { stale_fact_count } => {
            BankMutationStatus::Stale { stale_fact_count }
        }
        BankMutationCommitOutcome::Cancelled => BankMutationStatus::Cancelled,
        BankMutationCommitOutcome::TimedOut => BankMutationStatus::TimedOut,
        BankMutationCommitOutcome::Denied { kind, stage } => {
            return denied(BankMutationDenial::Commit { kind, stage }, work)
        }
        BankMutationCommitOutcome::CustomInvariantDenied { denial, stage } => {
            return denied(BankMutationDenial::CustomInvariant { denial, stage }, work)
        }
        BankMutationCommitOutcome::Aborted => BankMutationStatus::Aborted,
        BankMutationCommitOutcome::Deferred(deferred) => BankMutationStatus::Deferred(deferred),
        BankMutationCommitOutcome::SettlementDeferred(deferred) => {
            BankMutationStatus::SettlementDeferred(deferred)
        }
        BankMutationCommitOutcome::Indeterminate(evidence) => {
            BankMutationStatus::Indeterminate(evidence)
        }
    };
    BankMutationOutcome::new(status, work)
}

fn denied(
    denial: BankMutationDenial,
    work: Option<BankMutationProjectionWork>,
) -> BankMutationOutcome {
    BankMutationOutcome::new(BankMutationStatus::Denied(denial), work)
}

fn map_admission_denial(denial: BankOperationAdmissionError) -> BankMutationDenial {
    match denial {
        BankOperationAdmissionError::ProductSelection(denial) => {
            BankMutationDenial::ProductSelection(denial)
        }
        BankOperationAdmissionError::ScopeResolution(denial) => BankMutationDenial::Scope(denial),
        BankOperationAdmissionError::OperationInstallation(denial) => {
            BankMutationDenial::Installation(denial)
        }
        BankOperationAdmissionError::Authorization(denial) => {
            BankMutationDenial::Authorization(denial)
        }
    }
}
