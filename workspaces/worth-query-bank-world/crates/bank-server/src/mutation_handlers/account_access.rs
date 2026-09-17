use bank_domain::proposals::{
    BankAccountAuthorization, BankProposalDenial, BankProposalEngine, BankProposedEffect,
};
use bank_domain::schema::{
    AccountAccessResult, AccountAuthorization, AccountAuthorizationIdentity, AccountAuthorizedUser,
    AccountIdentity, AuthorizationAccount, AuthorizationRole, BankSchema,
    GrantAccountAccessMutationBinding, GrantAccountAuthorization, PrincipalIdentityField,
    RevokeAccountAccessMutationBinding, RevokeAccountAuthorization,
    ACCOUNT_ACCESS_OUTPUT_AUTHORIZATION,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    WorthQueryRetireOutput,
};

use crate::bank_projection::{
    project_account_authorization_grant, project_account_authorization_revoke,
};
use crate::graph_bootstrap::authorization_key;
use crate::operation_admission::bank_operation_scope_binding;

pub(crate) struct GrantAccountAccessHandler;
pub(crate) struct RevokeAccountAccessHandler;

impl OperationHandler<BankSchema, GrantAccountAccessMutationBinding> for GrantAccountAccessHandler {
    fn decide(
        &self,
        input: &GrantAccountAuthorization,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, GrantAccountAccessMutationBinding>,
    ) -> HandlerResult<bank_domain::proposals::BankInvariantApprovedProposal, BankProposalDenial>
    {
        let account = reader.scope().clone();
        let snapshot = match project_account_authorization_grant(reader.reader(), &account, input) {
            Ok(snapshot) => snapshot,
            Err(error) => return execution_denied(error),
        };
        match BankProposalEngine::prepare_grant_account_authorization(
            &snapshot,
            bank_operation_scope_binding(reader.operation_scope_binding()),
            reader.idempotency_key(),
            input,
        ) {
            Ok(proposal) => HandlerResult::Completed(proposal),
            Err(denial) => HandlerResult::DomainDenied(denial),
        }
    }

    fn candidate_requirements(
        &self,
        _: &GrantAccountAuthorization,
        _: &bank_domain::proposals::BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        GrantAccountAccessMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &GrantAccountAuthorization,
        decision: bank_domain::proposals::BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, GrantAccountAccessMutationBinding>,
    ) -> HandlerResult<AccountAccessResult, BankProposalDenial> {
        match exact_grant(decision.effects()).and_then(|grant| author_grant(grant, candidate)) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

impl OperationHandler<BankSchema, RevokeAccountAccessMutationBinding>
    for RevokeAccountAccessHandler
{
    fn decide(
        &self,
        input: &RevokeAccountAuthorization,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, RevokeAccountAccessMutationBinding>,
    ) -> HandlerResult<bank_domain::proposals::BankInvariantApprovedProposal, BankProposalDenial>
    {
        let account = reader.scope().clone();
        let snapshot = match project_account_authorization_revoke(
            reader.reader(),
            &account,
            input.account,
            input,
        ) {
            Ok(snapshot) => snapshot,
            Err(error) => return execution_denied(error),
        };
        match BankProposalEngine::prepare_revoke_account_authorization(
            &snapshot,
            bank_operation_scope_binding(reader.operation_scope_binding()),
            reader.idempotency_key(),
            input,
        ) {
            Ok(proposal) => HandlerResult::Completed(proposal),
            Err(denial) => HandlerResult::DomainDenied(denial),
        }
    }

    fn candidate_requirements(
        &self,
        _: &RevokeAccountAuthorization,
        _: &bank_domain::proposals::BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        RevokeAccountAccessMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &RevokeAccountAuthorization,
        decision: bank_domain::proposals::BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, RevokeAccountAccessMutationBinding>,
    ) -> HandlerResult<AccountAccessResult, BankProposalDenial> {
        match exact_revoke(decision.effects()).and_then(|revoke| author_revoke(revoke, candidate)) {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

fn author_grant(
    authorization: BankAccountAuthorization,
    candidate: &mut CandidateWriter<'_, BankSchema, GrantAccountAccessMutationBinding>,
) -> Result<AccountAccessResult, HandlerExecutionDenial> {
    let account = candidate
        .resolve_entity(AccountIdentity::reference(), authorization.account())
        .map_err(HandlerExecutionDenial::new)?;
    let principal = candidate
        .resolve_entity(
            PrincipalIdentityField::reference(),
            authorization.principal(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    let created = candidate
        .create_entity_in_context(
            &account,
            AccountAuthorization::reference(),
            WorthQueryApplicationEntityKey::new(authorization_key(authorization.id()))
                .map_err(HandlerExecutionDenial::new)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &created,
            AccountAuthorizationIdentity::reference(),
            authorization.id(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &created,
            AuthorizationRole::reference(),
            authorization.role(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            AccountAuthorizedUser::reference(),
            format!("authorized-user:{}", authorization.id().canonical_text()),
            &principal,
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            AuthorizationAccount::reference(),
            format!(
                "authorization-account:{}",
                authorization.id().canonical_text()
            ),
            &created,
            &account,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .create_output(
            WorthQueryApplicationOutputRole::<
                GrantAccountAccessMutationBinding,
                AccountAuthorization,
                WorthQueryCreateOutput,
            >::from_static(ACCOUNT_ACCESS_OUTPUT_AUTHORIZATION),
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(AccountAccessResult {
        authorization: authorization.id(),
    })
}

fn author_revoke(
    authorization: BankAccountAuthorization,
    candidate: &mut CandidateWriter<'_, BankSchema, RevokeAccountAccessMutationBinding>,
) -> Result<AccountAccessResult, HandlerExecutionDenial> {
    let account = candidate
        .resolve_entity(AccountIdentity::reference(), authorization.account())
        .map_err(HandlerExecutionDenial::new)?;
    let principal = candidate
        .resolve_entity(
            PrincipalIdentityField::reference(),
            authorization.principal(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    let authorization_entity = candidate
        .resolve_entity(
            AccountAuthorizationIdentity::reference(),
            authorization.id(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .unlink(
            AccountAuthorizedUser::reference(),
            &principal,
            &authorization_entity,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .unlink(
            AuthorizationAccount::reference(),
            &authorization_entity,
            &account,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .retire_output(
            WorthQueryApplicationOutputRole::<
                RevokeAccountAccessMutationBinding,
                AccountAuthorization,
                WorthQueryRetireOutput,
            >::from_static(ACCOUNT_ACCESS_OUTPUT_AUTHORIZATION),
            &authorization_entity,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .delete_entity(AccountAuthorization::reference(), &authorization_entity)
        .map_err(HandlerExecutionDenial::new)?;
    Ok(AccountAccessResult {
        authorization: authorization.id(),
    })
}

fn exact_grant(
    effects: &[BankProposedEffect],
) -> Result<BankAccountAuthorization, HandlerExecutionDenial> {
    let [BankProposedEffect::GrantAuthorization(authorization)] = effects else {
        return Err(HandlerExecutionDenial::new(InvalidAccountAccessCandidate));
    };
    Ok(*authorization)
}

fn exact_revoke(
    effects: &[BankProposedEffect],
) -> Result<BankAccountAuthorization, HandlerExecutionDenial> {
    let [BankProposedEffect::RevokeAuthorization(authorization)] = effects else {
        return Err(HandlerExecutionDenial::new(InvalidAccountAccessCandidate));
    };
    Ok(*authorization)
}

fn execution_denied<Value, Denial>(
    error: impl std::error::Error + Send + Sync + 'static,
) -> HandlerResult<Value, Denial> {
    HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
}

#[derive(Debug)]
struct InvalidAccountAccessCandidate;

impl std::fmt::Display for InvalidAccountAccessCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("account access proposal has an invalid effect shape")
    }
}

impl std::error::Error for InvalidAccountAccessCandidate {}
