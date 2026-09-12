use bank_domain::model::AccountJournalRevision;
use bank_domain::proposals::BankProposalDenial;
use bank_domain::schema::{
    Account, AccountDisplayName, AccountIdentity, AccountKind, AccountStatus, AccountingRevision,
    BankSchema, CreatePersonalAccount, CreatePersonalAccountDecision,
    CreatePersonalAccountMutationBinding, CreatePersonalAccountResult, InstitutionAccount,
    InstitutionIdentityField, Kind, PersonalOwner, PrincipalIdentityField, Status,
    CREATE_PERSONAL_ACCOUNT_OUTPUT_ACCOUNT,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerInterruption, HandlerResult,
    OperationHandler, WorthQueryApplicationEntityKey, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput,
};

use crate::bank_projection::project_personal_account_creation;
use crate::graph_bootstrap::account_key;

pub(crate) struct CreatePersonalAccountHandler;

impl OperationHandler<BankSchema, CreatePersonalAccountMutationBinding>
    for CreatePersonalAccountHandler
{
    fn decide(
        &self,
        input: &CreatePersonalAccount,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, CreatePersonalAccountMutationBinding>,
    ) -> HandlerResult<CreatePersonalAccountDecision, BankProposalDenial> {
        if let Err(interruption) = reader.checkpoint() {
            return interrupted(interruption);
        }
        let scope = reader.scope().clone();
        let key = reader.idempotency_key().clone();
        let snapshot = match project_personal_account_creation(reader.reader(), &scope, input) {
            Ok(snapshot) => snapshot,
            Err(error) => return execution_denied(error),
        };
        if !snapshot.is_known_institution(input.institution) {
            return HandlerResult::DomainDenied(BankProposalDenial::UnknownInstitution);
        }
        if !snapshot.is_known_principal(input.owner) {
            return HandlerResult::DomainDenied(BankProposalDenial::UnknownPrincipal);
        }
        if snapshot.primary_account(input.owner).is_some() {
            return HandlerResult::DomainDenied(BankProposalDenial::DuplicatePersonalAccount);
        }
        HandlerResult::Completed(CreatePersonalAccountDecision::from_admitted_input(
            &key, input,
        ))
    }

    fn candidate_requirements(
        &self,
        _input: &CreatePersonalAccount,
        _decision: &CreatePersonalAccountDecision,
    ) -> ApplicationCandidateRequirements {
        CreatePersonalAccountMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _input: &CreatePersonalAccount,
        decision: CreatePersonalAccountDecision,
        candidate: &mut CandidateWriter<'_, BankSchema, CreatePersonalAccountMutationBinding>,
    ) -> HandlerResult<CreatePersonalAccountResult, BankProposalDenial> {
        if let Err(interruption) = candidate.checkpoint() {
            return interrupted(interruption);
        }
        match author_candidate(decision, candidate) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

fn author_candidate(
    decision: CreatePersonalAccountDecision,
    candidate: &mut CandidateWriter<'_, BankSchema, CreatePersonalAccountMutationBinding>,
) -> Result<CreatePersonalAccountResult, HandlerExecutionDenial> {
    let (account_id, institution_id, owner_id, display_name) = decision.into_parts();
    let institution = candidate
        .resolve_entity(InstitutionIdentityField::reference(), institution_id)
        .map_err(HandlerExecutionDenial::new)?;
    let owner = candidate
        .resolve_entity(PrincipalIdentityField::reference(), owner_id)
        .map_err(HandlerExecutionDenial::new)?;
    let key = WorthQueryApplicationEntityKey::<BankSchema, Account>::new(
        account_key(account_id).into_boxed_str().into_string(),
    )
    .map_err(HandlerExecutionDenial::new)?;
    let created = candidate
        .candidate()
        .create_entity(Account::reference(), key)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .candidate()
        .initialize_field(&created, AccountIdentity::reference(), account_id)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .candidate()
        .initialize_field(&created, AccountDisplayName::reference(), display_name)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .candidate()
        .initialize_field(
            &created,
            AccountingRevision::reference(),
            AccountJournalRevision::default(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .candidate()
        .initialize_field(&created, Kind::reference(), AccountKind::Personal)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .candidate()
        .initialize_field(&created, Status::reference(), AccountStatus::Open)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .candidate()
        .link(
            InstitutionAccount::reference(),
            format!("institution-account:{}", account_id.canonical_text())
                .into_boxed_str()
                .into_string(),
            &institution,
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .candidate()
        .link(
            PersonalOwner::reference(),
            format!("personal-owner:{}", account_id.canonical_text())
                .into_boxed_str()
                .into_string(),
            &owner,
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .create_output(
            WorthQueryApplicationOutputRole::<
                CreatePersonalAccountMutationBinding,
                Account,
                WorthQueryCreateOutput,
            >::new(CREATE_PERSONAL_ACCOUNT_OUTPUT_ACCOUNT),
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(CreatePersonalAccountResult {
        account: account_id,
    })
}

fn execution_denied<Value, Denial>(
    error: impl std::error::Error + Send + Sync + 'static,
) -> HandlerResult<Value, Denial> {
    HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
}

fn interrupted<Value, Denial>(interruption: HandlerInterruption) -> HandlerResult<Value, Denial> {
    match interruption {
        HandlerInterruption::Cancelled => HandlerResult::Cancelled,
        HandlerInterruption::DeadlineExceeded => HandlerResult::DeadlineExceeded,
    }
}
