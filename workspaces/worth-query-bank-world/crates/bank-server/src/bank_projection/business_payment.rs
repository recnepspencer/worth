use bank_domain::model::BankPrincipalId;
use bank_domain::proposals::{BankDecisionSnapshot, BankSnapshot};
use bank_domain::schema::{
    ApprovePayment, ApprovePaymentOperation, InitiateBusinessPayment,
    InitiateBusinessPaymentOperation, RejectPayment, RejectPaymentOperation,
};

use super::bounded::{BoundedProjectionState, BusinessEntity, PaymentEntity, ProjectionReader};
use super::{account_balance::validated_account_balance, BankProjectionDenial};

pub(crate) fn project_business_payment_initiation(
    reader: &mut ProjectionReader<'_, '_, InitiateBusinessPaymentOperation>,
    business: &BusinessEntity,
    initiator: BankPrincipalId,
    input: &InitiateBusinessPayment,
) -> Result<BankSnapshot, BankProjectionDenial> {
    let mut state = BoundedProjectionState::new(reader)?;
    state.project_admitted_business(reader, business, input.business)?;
    if let Some(account) = state.project_business_account(reader, business)? {
        state.project_account_authorizations(reader, &account)?;
    }
    state.project_principal(reader, initiator)?;
    let recipient = state.project_principal(reader, input.recipient)?;
    state.project_primary_account(reader, &recipient)?;
    state
        .finish()
        .build()
        .map_err(BankProjectionDenial::InvalidDomainState)
}

pub(crate) fn project_payment_approval(
    reader: &mut ProjectionReader<'_, '_, ApprovePaymentOperation>,
    payment: &PaymentEntity,
    input: &ApprovePayment,
) -> Result<BankDecisionSnapshot, BankProjectionDenial> {
    let mut state = BoundedProjectionState::new(reader)?;
    let accounts = state.project_admitted_payment(reader, payment, input.payment)?;
    state.project_principal(reader, input.approver)?;
    let source_balance = validated_account_balance(
        accounts.source_id(),
        accounts.source_revision(),
        reader.summarize_exclusive_incoming(
            bank_domain::schema::PostingAccount::reference(),
            bank_domain::schema::PostingAmount::reference(),
            accounts.source(),
        )?,
    )?;
    state
        .finish()
        .build_decision_projection_with_balances(
            [accounts.source_id()],
            [(accounts.source_id(), source_balance)],
        )
        .map_err(BankProjectionDenial::InvalidDomainState)
}

pub(crate) fn project_payment_rejection(
    reader: &mut ProjectionReader<'_, '_, RejectPaymentOperation>,
    payment: &PaymentEntity,
    input: &RejectPayment,
) -> Result<BankDecisionSnapshot, BankProjectionDenial> {
    let mut state = BoundedProjectionState::new(reader)?;
    state.project_admitted_payment(reader, payment, input.payment)?;
    state.project_principal(reader, input.rejecting_principal)?;
    state
        .finish()
        .build_decision_projection([])
        .map_err(BankProjectionDenial::InvalidDomainState)
}
