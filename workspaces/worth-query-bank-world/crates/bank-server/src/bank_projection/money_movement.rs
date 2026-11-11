use bank_domain::model::{AccountId, InstitutionId};
use bank_domain::proposals::BankDecisionSnapshot;
use bank_domain::schema::*;
use worth_query_host::facade::domain::OperationReads;

use super::bounded::{BoundedProjectionState, InstitutionEntity, ProjectionReader};
use super::{account_balance::validated_account_balance, BankProjectionDenial};

pub(crate) fn project_institution_money_movement<Operation>(
    reader: &mut ProjectionReader<'_, '_, Operation>,
    institution: &InstitutionEntity,
    institution_id: InstitutionId,
    account_id: AccountId,
    required_balance_accounts: impl IntoIterator<Item = AccountId>,
) -> Result<BankDecisionSnapshot, BankProjectionDenial>
where
    InstitutionAccount: OperationReads<Operation>,
    InstitutionCashAccount: OperationReads<Operation>,

    PersonalOwner: OperationReads<Operation>,
    BusinessAccount: OperationReads<Operation>,

    PostingAccount: OperationReads<Operation>,
    JournalPosting: OperationReads<Operation>,

    JournalReversal: OperationReads<Operation>,
{
    let mut state = BoundedProjectionState::new(reader)?;
    state.project_admitted_institution(reader, institution, institution_id)?;
    let account = state.project_account_by_id(reader, account_id)?;
    let account_revision = state.project_account(reader, &account)?;
    let cash = state.project_institution_cash_account(reader, institution)?;
    let cash_revision = state.project_account(reader, &cash)?;
    let cash_id = reader
        .decision_field(&cash, AccountIdentity::at("AccountIdentity"))?
        .ok_or(BankProjectionDenial::MissingField("AccountIdentity"))?;
    let account_balance = validated_account_balance(
        account_id,
        account_revision,
        reader.summarize_exclusive_incoming(
            PostingAccount::reference(),
            PostingAmount::at("PostingAmount"),
            &account,
        )?,
    )?;
    let cash_balance = validated_account_balance(
        cash_id,
        cash_revision,
        reader.summarize_exclusive_incoming(
            PostingAccount::reference(),
            PostingAmount::at("PostingAmount"),
            &cash,
        )?,
    )?;
    state
        .finish()
        .build_decision_projection_with_balances(
            required_balance_accounts,
            [(account_id, account_balance), (cash_id, cash_balance)],
        )
        .map_err(BankProjectionDenial::InvalidDomainState)
}
