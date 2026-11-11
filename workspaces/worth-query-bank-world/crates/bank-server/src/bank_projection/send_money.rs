use bank_domain::proposals::BankDecisionSnapshot;
use bank_domain::schema::*;

use super::bounded::{AccountEntity, BoundedProjectionState, ProjectionReader};
use super::{account_balance::validated_account_balance, missing_field, BankProjectionDenial};

pub(crate) fn project_send_money_decision(
    reader: &mut ProjectionReader<'_, '_, SendMoneyOperation>,
    source: &AccountEntity,
    input: &SendMoney,
) -> Result<BankDecisionSnapshot, BankProjectionDenial> {
    let recipient = reader.resolve_entity(
        PrincipalIdentityField::at("PrincipalIdentityField"),
        input.recipient,
    )?;
    let destinations = reader.relations_from(PersonalOwner::reference(), &recipient)?;
    let [destination_relation] = destinations.as_slice() else {
        return Err(if destinations.is_empty() {
            BankProjectionDenial::MissingRelation("PersonalOwner")
        } else {
            BankProjectionDenial::AmbiguousRelation("PersonalOwner")
        });
    };
    let destination = destination_relation.to().clone();
    let destination_id = missing_field(
        reader.decision_field(&destination, AccountIdentity::at("AccountIdentity"))?,
        "AccountIdentity",
    )?;

    let mut state = BoundedProjectionState::for_capability_projection(reader)?;
    let source_revision = state.project_admitted_account(reader, source, input.from)?;
    let destination_revision = state.project_account(reader, &destination)?;
    let source_balance = validated_account_balance(
        input.from,
        source_revision,
        reader.summarize_exclusive_incoming(
            PostingAccount::reference(),
            PostingAmount::at("PostingAmount"),
            source,
        )?,
    )?;
    let destination_balance = validated_account_balance(
        destination_id,
        destination_revision,
        reader.summarize_exclusive_incoming(
            PostingAccount::reference(),
            PostingAmount::at("PostingAmount"),
            &destination,
        )?,
    )?;
    reader.require_decision_field(source, AccountIdentity::at("AccountIdentity"))?;
    reader.require_decision_field(&destination, AccountIdentity::at("AccountIdentity"))?;
    reader.require_decision_field(
        &recipient,
        PrincipalIdentityField::at("PrincipalIdentityField"),
    )?;
    reader.require_decision_relation(PersonalOwner::reference(), &recipient, &destination)?;
    reader.require_decision_field(source, AccountingRevision::at("AccountingRevision"))?;
    reader.require_decision_field(&destination, AccountingRevision::at("AccountingRevision"))?;
    reader.require_decision_field(source, Status::at("Status"))?;
    reader.require_decision_field(&destination, Status::at("Status"))?;
    state
        .finish()
        .build_decision_projection_with_balances(
            [input.from],
            [
                (input.from, source_balance),
                (destination_id, destination_balance),
            ],
        )
        .map_err(BankProjectionDenial::InvalidDomainState)
}
