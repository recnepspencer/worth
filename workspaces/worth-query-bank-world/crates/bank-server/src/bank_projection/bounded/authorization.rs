use bank_domain::model::{AccountAuthorizationId, AccountId};
use bank_domain::proposals::BankAccountAuthorization;
use bank_domain::schema::*;
use worth_query_host::facade::domain::OperationReads;

use super::{
    AccountEntity, AuthorizationEntity, BoundedProjectionState, PrincipalEntity, ProjectionReader,
};
use crate::bank_projection::{missing_field, BankProjectionDenial};

impl BoundedProjectionState {
    pub(in crate::bank_projection) fn project_authorization_by_id<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        id: AccountAuthorizationId,
    ) -> Result<AuthorizationEntity, BankProjectionDenial>
    where
        AccountAuthorization: OperationReads<Operation>,
        AccountAuthorizationIdentity: OperationReads<Operation>,
        AccountAuthorizedUser: OperationReads<Operation>,
        AuthorizationAccount: OperationReads<Operation>,
        AuthorizationRole: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        AccountIdentity: OperationReads<Operation>,
        AccountingRevision: OperationReads<Operation>,
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
        Kind: OperationReads<Operation>,
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
        Status: OperationReads<Operation>,
        AccountDisplayName: OperationReads<Operation>,
    {
        let authorization = reader.resolve_entity(AccountAuthorizationIdentity::reference(), id)?;
        reader.require_decision_entity(&authorization, AccountAuthorization::reference())?;
        self.project_authorization(reader, &authorization)?;
        Ok(authorization)
    }

    pub(in crate::bank_projection) fn project_matching_authorization<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        principal: &PrincipalEntity,
        account: AccountId,
    ) -> Result<(), BankProjectionDenial>
    where
        AccountAuthorizedUser: OperationReads<Operation>,
        AuthorizationAccount: OperationReads<Operation>,
        AccountIdentity: OperationReads<Operation>,
        AccountAuthorizationIdentity: OperationReads<Operation>,
        AuthorizationRole: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        AccountingRevision: OperationReads<Operation>,
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
        Kind: OperationReads<Operation>,
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
        Status: OperationReads<Operation>,
        AccountDisplayName: OperationReads<Operation>,
    {
        let relations =
            reader.decision_relations_from(AccountAuthorizedUser::reference(), principal)?;
        let mut matching = Vec::new();
        for relation in relations {
            let authorization = relation.to();
            validate_authorization_principal(reader, authorization, principal)?;
            let account_relation = authorization_account(reader, authorization)?;
            let account_id = missing_field(
                reader.decision_field(account_relation.to(), AccountIdentity::reference())?,
                "AccountIdentity",
            )?;
            if account_id == account {
                matching.push(authorization.clone());
            }
        }
        match matching.as_slice() {
            [] => Ok(()),
            [authorization] => self.project_authorization(reader, authorization),
            _ => Err(BankProjectionDenial::AmbiguousRelation(
                "account authorization",
            )),
        }
    }

    /// Projects every authorization on one account, so a decision can see
    /// who holds which role there.
    pub(in crate::bank_projection) fn project_account_authorizations<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        account: &AccountEntity,
    ) -> Result<(), BankProjectionDenial>
    where
        AccountAuthorizationIdentity: OperationReads<Operation>,
        AccountAuthorizedUser: OperationReads<Operation>,
        AuthorizationAccount: OperationReads<Operation>,
        AuthorizationRole: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        AccountIdentity: OperationReads<Operation>,
        AccountingRevision: OperationReads<Operation>,
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
        Kind: OperationReads<Operation>,
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
        Status: OperationReads<Operation>,
        AccountDisplayName: OperationReads<Operation>,
    {
        let relations = reader.decision_relations_to(AuthorizationAccount::reference(), account)?;
        for relation in relations {
            self.project_authorization(reader, relation.from())?;
        }
        Ok(())
    }

    fn project_authorization<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        authorization: &AuthorizationEntity,
    ) -> Result<(), BankProjectionDenial>
    where
        AccountAuthorizationIdentity: OperationReads<Operation>,
        AccountAuthorizedUser: OperationReads<Operation>,
        AuthorizationAccount: OperationReads<Operation>,
        AuthorizationRole: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        AccountIdentity: OperationReads<Operation>,
        AccountingRevision: OperationReads<Operation>,
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
        Kind: OperationReads<Operation>,
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
        Status: OperationReads<Operation>,
        AccountDisplayName: OperationReads<Operation>,
    {
        let id = missing_field(
            reader.decision_field(authorization, AccountAuthorizationIdentity::reference())?,
            "AccountAuthorizationIdentity",
        )?;
        let canonical = reader.resolve_entity(AccountAuthorizationIdentity::reference(), id)?;
        if &canonical != authorization {
            return Err(BankProjectionDenial::AmbiguousRelation(
                "AccountAuthorizationIdentity",
            ));
        }
        let principal_relations =
            reader.decision_relations_to(AccountAuthorizedUser::reference(), authorization)?;
        let principal_relation = exactly_one(&principal_relations, "AccountAuthorizedUser")?;
        let principal_id = missing_field(
            reader.decision_field(
                principal_relation.from(),
                PrincipalIdentityField::reference(),
            )?,
            "PrincipalIdentityField",
        )?;
        self.project_principal(reader, principal_id)?;
        let account_relation = authorization_account(reader, authorization)?;
        let account_id = missing_field(
            reader.decision_field(account_relation.to(), AccountIdentity::reference())?,
            "AccountIdentity",
        )?;
        self.project_account(reader, account_relation.to())?;
        let role = missing_field(
            reader.decision_field(authorization, AuthorizationRole::reference())?,
            "AuthorizationRole",
        )?;
        self.update_builder(|builder| {
            builder.projected_authorization(BankAccountAuthorization::from_projection(
                id,
                account_id,
                principal_id,
                role,
            ))
        });
        Ok(())
    }
}

fn validate_authorization_principal<Operation>(
    reader: &mut ProjectionReader<'_, '_, Operation>,
    authorization: &AuthorizationEntity,
    principal: &PrincipalEntity,
) -> Result<(), BankProjectionDenial>
where
    AccountAuthorizedUser: OperationReads<Operation>,
{
    let relations =
        reader.decision_relations_to(AccountAuthorizedUser::reference(), authorization)?;
    if matches!(relations.as_slice(), [relation] if relation.from() == principal) {
        Ok(())
    } else {
        Err(BankProjectionDenial::AmbiguousRelation(
            "AccountAuthorizedUser",
        ))
    }
}

fn authorization_account<Operation>(
    reader: &mut ProjectionReader<'_, '_, Operation>,
    authorization: &AuthorizationEntity,
) -> Result<
    worth_query_host::facade::primary_graph::WorthQueryInvariantRelation<
        BankSchema,
        AuthorizationAccount,
        AccountAuthorization,
        Account,
    >,
    BankProjectionDenial,
>
where
    AuthorizationAccount: OperationReads<Operation>,
{
    let relations =
        reader.decision_relations_from(AuthorizationAccount::reference(), authorization)?;
    exactly_one(&relations, "AuthorizationAccount").cloned()
}

fn exactly_one<'row, Row>(
    rows: &'row [Row],
    relation: &'static str,
) -> Result<&'row Row, BankProjectionDenial> {
    match rows {
        [row] => Ok(row),
        [] => Err(BankProjectionDenial::MissingRelation(relation)),
        _ => Err(BankProjectionDenial::AmbiguousRelation(relation)),
    }
}
