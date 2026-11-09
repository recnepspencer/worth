use bank_domain::model::{AccountId, AccountJournalRevision, PaymentId};
use bank_domain::payments::{BusinessPayment, BusinessPaymentProjection};
use bank_domain::schema::*;
use worth_query_host::facade::domain::OperationReads;

use super::{
    AccountEntity, ApprovalEntity, BoundedProjectionState, PaymentEntity, ProjectionReader,
};
use crate::bank_projection::{missing_field, BankProjectionDenial};

pub(in crate::bank_projection) struct ProjectedPaymentAccounts {
    source: AccountEntity,
    source_id: AccountId,
    source_revision: AccountJournalRevision,
}

struct PaymentAccountProjection {
    source: AccountEntity,
    source_id: AccountId,
    source_revision: AccountJournalRevision,
    destination_id: AccountId,
}

impl ProjectedPaymentAccounts {
    pub(in crate::bank_projection) const fn source_id(&self) -> AccountId {
        self.source_id
    }

    pub(in crate::bank_projection) const fn source(&self) -> &AccountEntity {
        &self.source
    }

    pub(in crate::bank_projection) const fn source_revision(&self) -> AccountJournalRevision {
        self.source_revision
    }
}

impl BoundedProjectionState {
    pub(in crate::bank_projection) fn project_admitted_payment<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        payment: &PaymentEntity,
        id: PaymentId,
    ) -> Result<ProjectedPaymentAccounts, BankProjectionDenial>
    where
        PaymentSource: OperationReads<Operation>,
        PaymentDestination: OperationReads<Operation>,
        PaymentBusiness: OperationReads<Operation>,
        PaymentInitiator: OperationReads<Operation>,
        PaymentApproval: OperationReads<Operation>,
        ApprovalPrincipal: OperationReads<Operation>,

        InstitutionAccount: OperationReads<Operation>,

        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
    {
        self.project_payment(reader, payment, id)
    }

    fn project_payment<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        payment: &PaymentEntity,
        id: PaymentId,
    ) -> Result<ProjectedPaymentAccounts, BankProjectionDenial>
    where
        PaymentSource: OperationReads<Operation>,
        PaymentDestination: OperationReads<Operation>,
        PaymentBusiness: OperationReads<Operation>,
        PaymentInitiator: OperationReads<Operation>,
        PaymentApproval: OperationReads<Operation>,
        ApprovalPrincipal: OperationReads<Operation>,

        InstitutionAccount: OperationReads<Operation>,

        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
    {
        self.validate_payment_identity(reader, payment, id)?;
        let accounts = self.project_payment_accounts(reader, payment)?;
        let business_id = self.project_payment_business(reader, payment)?;
        let initiator_id = self.project_payment_initiator(reader, payment)?;
        let deciding_principal = self.project_payment_decision(reader, payment)?;

        let amount = missing_field(
            reader.decision_field(payment, PaymentAmount::at("PaymentAmount"))?,
            "PaymentAmount",
        )?;
        let status = missing_field(
            reader.decision_field(payment, PaymentStatusField::at("PaymentStatusField"))?,
            "PaymentStatusField",
        )?;
        self.update_builder(|builder| {
            builder.projected_payment(BusinessPayment::from_projection(
                BusinessPaymentProjection {
                    id,
                    business: business_id,
                    source: accounts.source_id,
                    destination: accounts.destination_id,
                    initiator: initiator_id,
                    amount,
                    status,
                    deciding_principal,
                },
            ))
        });
        Ok(ProjectedPaymentAccounts {
            source: accounts.source,
            source_id: accounts.source_id,
            source_revision: accounts.source_revision,
        })
    }

    fn project_payment_accounts<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        payment: &PaymentEntity,
    ) -> Result<PaymentAccountProjection, BankProjectionDenial>
    where
        PaymentSource: OperationReads<Operation>,
        PaymentDestination: OperationReads<Operation>,

        InstitutionAccount: OperationReads<Operation>,

        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,
    {
        let source = exactly_one(
            &reader.decision_relations_from(PaymentSource::reference(), payment)?,
            "PaymentSource",
        )?
        .to()
        .clone();
        let destination = exactly_one(
            &reader.decision_relations_from(PaymentDestination::reference(), payment)?,
            "PaymentDestination",
        )?
        .to()
        .clone();
        let source_id = required_account_id(reader, &source)?;
        let destination_id = required_account_id(reader, &destination)?;
        let source_revision = self.project_account(reader, &source)?;
        self.project_account(reader, &destination)?;
        Ok(PaymentAccountProjection {
            source,
            source_id,
            source_revision,
            destination_id,
        })
    }

    fn project_payment_business<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        payment: &PaymentEntity,
    ) -> Result<bank_domain::model::BusinessId, BankProjectionDenial>
    where
        PaymentBusiness: OperationReads<Operation>,
    {
        let relations = reader.decision_relations_from(PaymentBusiness::reference(), payment)?;
        let business = exactly_one(&relations, "PaymentBusiness")?.to();
        let id = missing_field(
            reader.decision_field(business, BusinessIdentityField::at("BusinessIdentityField"))?,
            "BusinessIdentityField",
        )?;
        self.project_business(reader, id)?;
        Ok(id)
    }

    fn project_payment_initiator<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        payment: &PaymentEntity,
    ) -> Result<bank_domain::model::BankPrincipalId, BankProjectionDenial>
    where
        PaymentInitiator: OperationReads<Operation>,
    {
        let relations = reader.decision_relations_to(PaymentInitiator::reference(), payment)?;
        let initiator = exactly_one(&relations, "PaymentInitiator")?.from();
        let id = missing_field(
            reader.decision_field(
                initiator,
                PrincipalIdentityField::at("PrincipalIdentityField"),
            )?,
            "PrincipalIdentityField",
        )?;
        self.project_principal(reader, id)?;
        Ok(id)
    }

    fn validate_payment_identity<Operation>(
        &self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        payment: &PaymentEntity,
        expected: PaymentId,
    ) -> Result<(), BankProjectionDenial>
where {
        let actual = missing_field(
            reader.decision_field(payment, PaymentIdentityField::at("PaymentIdentityField"))?,
            "PaymentIdentityField",
        )?;
        if actual == expected {
            Ok(())
        } else {
            Err(BankProjectionDenial::AmbiguousRelation(
                "PaymentIdentityField",
            ))
        }
    }

    fn project_payment_decision<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        payment: &PaymentEntity,
    ) -> Result<Option<bank_domain::model::BankPrincipalId>, BankProjectionDenial>
    where
        PaymentApproval: OperationReads<Operation>,
        ApprovalPrincipal: OperationReads<Operation>,
    {
        let approvals = reader.decision_relations_from(PaymentApproval::reference(), payment)?;
        let approval = match approvals.as_slice() {
            [] => return Ok(None),
            [approval] => approval.to(),
            _ => return Err(BankProjectionDenial::AmbiguousRelation("PaymentApproval")),
        };
        validate_approval_owner(reader, payment, approval)?;
        let principal_relations =
            reader.decision_relations_from(ApprovalPrincipal::reference(), approval)?;
        let principal = exactly_one(&principal_relations, "ApprovalPrincipal")?.to();
        let principal_id = missing_field(
            reader.decision_field(
                principal,
                PrincipalIdentityField::at("PrincipalIdentityField"),
            )?,
            "PrincipalIdentityField",
        )?;
        self.project_principal(reader, principal_id)?;
        Ok(Some(principal_id))
    }
}

fn validate_approval_owner<Operation>(
    reader: &mut ProjectionReader<'_, '_, Operation>,
    payment: &PaymentEntity,
    approval: &ApprovalEntity,
) -> Result<(), BankProjectionDenial>
where
    PaymentApproval: OperationReads<Operation>,
{
    let owners = reader.decision_relations_to(PaymentApproval::reference(), approval)?;
    if matches!(owners.as_slice(), [owner] if owner.from() == payment) {
        Ok(())
    } else {
        Err(BankProjectionDenial::AmbiguousRelation("PaymentApproval"))
    }
}

fn required_account_id<Operation>(
    reader: &mut ProjectionReader<'_, '_, Operation>,
    account: &AccountEntity,
) -> Result<AccountId, BankProjectionDenial>
where
{
    missing_field(
        reader.decision_field(account, AccountIdentity::at("AccountIdentity"))?,
        "AccountIdentity",
    )
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
