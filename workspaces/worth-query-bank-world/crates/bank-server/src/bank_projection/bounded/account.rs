use bank_domain::accounting::{BankAccount, BankAccountProjection};
use bank_domain::model::AccountJournalRevision;
use bank_domain::schema::*;
use worth_query_host::facade::domain::OperationReads;

use super::{AccountEntity, BoundedProjectionState, ProjectionReader};
use crate::bank_projection::{missing_field, BankProjectionDenial};

impl BoundedProjectionState {
    pub(in crate::bank_projection) fn project_account<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        entity: &AccountEntity,
    ) -> Result<AccountJournalRevision, BankProjectionDenial>
    where
        AccountIdentity: OperationReads<Operation>,
        AccountingRevision: OperationReads<Operation>,
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
        Kind: OperationReads<Operation>,
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
        Status: OperationReads<Operation>,
        AccountDisplayName: OperationReads<Operation>,
    {
        self.project_account_with_identity(reader, entity, None)
    }

    pub(in crate::bank_projection) fn project_admitted_account<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        entity: &AccountEntity,
        expected: bank_domain::model::AccountId,
    ) -> Result<AccountJournalRevision, BankProjectionDenial>
    where
        AccountIdentity: OperationReads<Operation>,
        AccountingRevision: OperationReads<Operation>,
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
        Kind: OperationReads<Operation>,
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
        Status: OperationReads<Operation>,
        AccountDisplayName: OperationReads<Operation>,
    {
        self.project_account_with_identity(reader, entity, Some(expected))
    }

    fn project_account_with_identity<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        entity: &AccountEntity,
        admitted_identity: Option<bank_domain::model::AccountId>,
    ) -> Result<AccountJournalRevision, BankProjectionDenial>
    where
        AccountIdentity: OperationReads<Operation>,
        AccountingRevision: OperationReads<Operation>,
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
        Kind: OperationReads<Operation>,
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
        Status: OperationReads<Operation>,
        AccountDisplayName: OperationReads<Operation>,
    {
        let id = required_field(
            self.projected_field(reader, entity, AccountIdentity::reference())?,
            "AccountIdentity",
        )?;
        if admitted_identity.is_some_and(|expected| expected != id) {
            return Err(BankProjectionDenial::AmbiguousRelation("AccountIdentity"));
        }
        let revision = required_field(
            self.projected_field(reader, entity, AccountingRevision::reference())?,
            "AccountingRevision",
        )?;
        if let Some(projected) = self.accounts.get(&id) {
            if projected != entity {
                return Err(BankProjectionDenial::AmbiguousRelation("AccountIdentity"));
            }
            return Ok(revision);
        }
        if admitted_identity.is_none() {
            let canonical = reader.resolve_entity(AccountIdentity::reference(), id)?;
            if &canonical != entity {
                return Err(BankProjectionDenial::AmbiguousRelation("AccountIdentity"));
            }
        }
        let institution_id = self.project_account_institution(reader, entity)?;
        let kind = required_field(
            self.projected_field(reader, entity, Kind::reference())?,
            "Kind",
        )?;
        let (personal_owner, business_owner) =
            self.project_account_ownership(reader, entity, kind)?;
        let account = BankAccount::from_projection(BankAccountProjection {
            id,
            institution: institution_id,
            kind,
            status: required_field(
                self.projected_field(reader, entity, Status::reference())?,
                "Status",
            )?,
            display_name: required_field(
                self.projected_field(reader, entity, AccountDisplayName::reference())?,
                "AccountDisplayName",
            )?,
            personal_owner,
            business_owner,
        })
        .ok_or(BankProjectionDenial::MissingRelation("account ownership"))?;
        self.update_builder(|builder| {
            builder
                .institution(institution_id)
                .projected_account(account)
                .projected_account_revision(id, revision)
        });
        self.accounts.insert(id, entity.clone());
        Ok(revision)
    }

    fn project_account_institution<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        entity: &AccountEntity,
    ) -> Result<bank_domain::model::InstitutionId, BankProjectionDenial>
    where
        InstitutionAccount: OperationReads<Operation>,
        InstitutionIdentityField: OperationReads<Operation>,
    {
        let relations =
            self.projected_relations_to(reader, InstitutionAccount::reference(), entity)?;
        let institution = exactly_one(&relations, "InstitutionAccount")?.from();
        let id = required_field(
            self.projected_field(reader, institution, InstitutionIdentityField::reference())?,
            "InstitutionIdentityField",
        )?;
        let canonical = reader.resolve_entity(InstitutionIdentityField::reference(), id)?;
        if &canonical != institution {
            return Err(BankProjectionDenial::AmbiguousRelation(
                "InstitutionIdentityField",
            ));
        }
        Ok(id)
    }

    fn project_account_ownership<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        entity: &AccountEntity,
        kind: AccountKind,
    ) -> Result<
        (
            Option<bank_domain::model::BankPrincipalId>,
            Option<bank_domain::model::BusinessId>,
        ),
        BankProjectionDenial,
    >
    where
        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
        PrincipalIdentityField: OperationReads<Operation>,
        BusinessIdentityField: OperationReads<Operation>,
    {
        let personal = self.projected_relations_to(reader, PersonalOwner::reference(), entity)?;
        let business = self.projected_relations_to(reader, BusinessAccount::reference(), entity)?;
        Ok(match kind {
            AccountKind::Personal => {
                let owner = exactly_one(&personal, "PersonalOwner")?.from();
                if !business.is_empty() {
                    return Err(BankProjectionDenial::AmbiguousRelation("account ownership"));
                }
                let owner_id = required_field(
                    self.projected_field(reader, owner, PrincipalIdentityField::reference())?,
                    "PrincipalIdentityField",
                )?;
                let canonical_owner =
                    reader.resolve_entity(PrincipalIdentityField::reference(), owner_id)?;
                if &canonical_owner != owner {
                    return Err(BankProjectionDenial::AmbiguousRelation(
                        "PrincipalIdentityField",
                    ));
                }
                self.update_builder(|builder| builder.principal(owner_id));
                (Some(owner_id), None)
            }
            AccountKind::Business => {
                let owner = exactly_one(&business, "BusinessAccount")?.from();
                if !personal.is_empty() {
                    return Err(BankProjectionDenial::AmbiguousRelation("account ownership"));
                }
                let owner_id = required_field(
                    self.projected_field(reader, owner, BusinessIdentityField::reference())?,
                    "BusinessIdentityField",
                )?;
                let canonical_owner =
                    reader.resolve_entity(BusinessIdentityField::reference(), owner_id)?;
                if &canonical_owner != owner {
                    return Err(BankProjectionDenial::AmbiguousRelation(
                        "BusinessIdentityField",
                    ));
                }
                self.update_builder(|builder| builder.business(owner_id));
                (None, Some(owner_id))
            }
            AccountKind::InstitutionCash | AccountKind::InstitutionSettlement => {
                if !personal.is_empty() || !business.is_empty() {
                    return Err(BankProjectionDenial::AmbiguousRelation("account ownership"));
                }
                (None, None)
            }
        })
    }
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

fn required_field<Value>(
    value: Option<Value>,
    field: &'static str,
) -> Result<Value, BankProjectionDenial> {
    missing_field(value, field)
}
