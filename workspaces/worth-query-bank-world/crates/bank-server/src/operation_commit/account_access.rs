use bank_domain::proposals::{BankAccountAuthorization, BankProposedEffect};
use bank_domain::schema::*;

use super::{
    application_idempotency, entity_key, BankCommitPreparationDenial, BankMutationCommitOutcome,
};
use crate::graph_bootstrap::authorization_key;
use crate::{BankAuthorizedProposal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn commit_grant_account_access(
        &self,
        proposal: BankAuthorizedProposal<
            GrantAccountAuthorizationOperation,
            GrantAccountAuthorization,
            Account,
            bank_domain::model::AccountId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (admission, invariant, projection) = proposal.into_parts();
        let authorization = exact_grant(invariant.effects())?;
        let (_, account_id, query_admission) = admission.into_parts();
        if authorization.account() != account_id {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let account = reads.resolve_entity(AccountIdentity::reference(), account_id)?;
        let principal = reads.resolve_entity(
            PrincipalIdentityField::reference(),
            authorization.principal(),
        )?;
        let mut effects = reads
            .complete_projected_dependencies()?
            .begin_effect_program();
        let account = effects.existing_entity(&account)?;
        let created = effects.create_entity_in_context(
            &account,
            AccountAuthorization::reference(),
            entity_key(authorization_key(authorization.id()))?,
        )?;
        effects.initialize_field(
            &created,
            AccountAuthorizationIdentity::reference(),
            authorization.id(),
        )?;
        effects.initialize_field(
            &created,
            AuthorizationRole::reference(),
            authorization.role(),
        )?;
        let principal = effects.existing_entity(&principal)?;
        effects.link(
            AccountAuthorizedUser::reference(),
            format!("authorized-user:{}", authorization.id().canonical_text()),
            &principal,
            &created,
        )?;
        effects.link(
            AuthorizationAccount::reference(),
            format!(
                "authorization-account:{}",
                authorization.id().canonical_text()
            ),
            &created,
            &account,
        )?;
        commit(self, invariant, effects.finish()?)
    }

    pub fn commit_revoke_account_access(
        &self,
        proposal: BankAuthorizedProposal<
            RevokeAccountAuthorizationOperation,
            RevokeAccountAuthorization,
            Account,
            bank_domain::model::AccountId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (admission, invariant, projection) = proposal.into_parts();
        let authorization = exact_revoke(invariant.effects())?;
        let (_, account_id, query_admission) = admission.into_parts();
        if authorization.account() != account_id {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let account = reads.resolve_entity(AccountIdentity::reference(), account_id)?;
        let principal = reads.resolve_entity(
            PrincipalIdentityField::reference(),
            authorization.principal(),
        )?;
        let authorization_identity = reads.resolve_entity(
            AccountAuthorizationIdentity::reference(),
            authorization.id(),
        )?;
        let completed = reads.complete_projected_dependencies()?;
        let authorized_user = completed.projected_relation(
            AccountAuthorizedUser::reference(),
            &principal,
            &authorization_identity,
        )?;
        let authorization_account = completed.projected_relation(
            AuthorizationAccount::reference(),
            &authorization_identity,
            &account,
        )?;
        let mut effects = completed.begin_effect_program();
        effects.unlink(AccountAuthorizedUser::reference(), authorized_user)?;
        effects.unlink(AuthorizationAccount::reference(), authorization_account)?;
        let authorization_entity = effects.existing_entity(&authorization_identity)?;
        effects.delete_entity(AccountAuthorization::reference(), &authorization_entity)?;
        commit(self, invariant, effects.finish()?)
    }
}

fn exact_grant(
    effects: &[BankProposedEffect],
) -> Result<BankAccountAuthorization, BankCommitPreparationDenial> {
    let [BankProposedEffect::GrantAuthorization(authorization)] = effects else {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    };
    Ok(*authorization)
}

fn exact_revoke(
    effects: &[BankProposedEffect],
) -> Result<BankAccountAuthorization, BankCommitPreparationDenial> {
    let [BankProposedEffect::RevokeAuthorization(authorization)] = effects else {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    };
    Ok(*authorization)
}

fn commit<Operation, Input, Scope>(
    runtime: &BankIdentityRuntime,
    invariant: bank_domain::proposals::BankInvariantApprovedProposal,
    program: worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgram<
        BankSchema,
        Operation,
        Input,
        Scope,
    >,
) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial>
where
    Input: Clone + Send + Sync + 'static,
{
    let idempotency = application_idempotency(&invariant);
    Ok(runtime
        .application_runtime()
        .compare_and_commit_application(program, idempotency)
        .into())
}
