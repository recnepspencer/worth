use crate::model::BankPrincipalId;
use crate::payments::BusinessPayment;
use crate::schema::{
    ApprovePayment, InitiateBusinessPayment, InitiateBusinessPaymentDecision, PaymentStatus,
    PostingPurpose, RejectPayment, MAX_PAYMENT_APPROVAL_GRANTEES,
};

use super::BankProposalEngine;
use crate::proposals::{
    append_balanced_transfer, complete_decision_proposal, complete_proposal, ensure_open,
    BankDecisionSnapshot, BankIdempotencyClaim, BankIdempotencyKey, BankInvariantApprovedProposal,
    BankOperationScopeBinding, BankProposalDenial, BankProposedEffect, BankSnapshot,
    CanonicalProposalPayload,
};

impl BankProposalEngine {
    pub fn prepare_initiate_business_payment(
        snapshot: &BankSnapshot,
        binding: BankOperationScopeBinding,
        key: &BankIdempotencyKey,
        initiator: BankPrincipalId,
        input: &InitiateBusinessPayment,
    ) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
        let destination = validate_business_payment(snapshot, initiator, input)?;
        let payload = CanonicalProposalPayload::new("initiate-business-payment")
            .u64("initiator", initiator.get())
            .u64("business", input.business.get())
            .text("source", &input.from.canonical_text())
            .text("destination", &destination.canonical_text())
            .i64("amount-minor-units", input.amount.minor_units());
        let intent = BankIdempotencyClaim::derive(binding, key, payload);
        let payment = business_payment(intent, destination, initiator, input);
        let mut proposed = snapshot.clone();
        proposed.insert_payment(payment.clone());
        complete_proposal(
            snapshot,
            proposed,
            intent,
            vec![BankProposedEffect::CreatePayment(payment)],
        )
    }

    pub fn decide_initiate_business_payment_from_application(
        snapshot: &BankSnapshot,
        idempotency: BankIdempotencyClaim,
        initiator: BankPrincipalId,
        input: &InitiateBusinessPayment,
    ) -> Result<InitiateBusinessPaymentDecision, BankProposalDenial> {
        let destination = validate_business_payment(snapshot, initiator, input)?;
        let payment = business_payment(idempotency, destination, initiator, input);
        let grantees = snapshot.payment_approval_grantees(&payment);
        if grantees.len() > MAX_PAYMENT_APPROVAL_GRANTEES {
            return Err(BankProposalDenial::TooManyPaymentApprovers);
        }
        Ok(InitiateBusinessPaymentDecision::new(
            payment,
            grantees.into_iter().collect(),
        ))
    }

    pub fn prepare_approve_payment(
        snapshot: &BankSnapshot,
        binding: BankOperationScopeBinding,
        key: &BankIdempotencyKey,
        input: &ApprovePayment,
    ) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
        let payment = pending_payment(snapshot, input.payment)?;
        if payment.initiator() == input.approver {
            return Err(BankProposalDenial::SelfApproval);
        }
        if !snapshot.is_known_principal(input.approver) {
            return Err(BankProposalDenial::UnknownPrincipal);
        }

        let payload = CanonicalProposalPayload::new("approve-payment")
            .text("payment", &input.payment.canonical_text())
            .u64("approver", input.approver.get());
        let intent = BankIdempotencyClaim::derive(binding, key, payload);
        let mut proposed = snapshot.clone();
        let journal = append_balanced_transfer(
            &mut proposed,
            payment.source(),
            payment.destination(),
            payment.amount(),
            PostingPurpose::Transfer,
            None,
            intent.key(),
        )?;
        let replacement = payment.with_decision(PaymentStatus::Committed, input.approver);
        proposed.replace_payment(replacement.clone());
        let effects = vec![
            BankProposedEffect::AppendJournal(journal),
            BankProposedEffect::UpdatePayment {
                payment: input.payment,
                replacement,
            },
        ];
        complete_proposal(snapshot, proposed, intent, effects)
    }

    pub fn prepare_approve_payment_from_decision(
        decision: BankDecisionSnapshot,
        binding: BankOperationScopeBinding,
        key: &BankIdempotencyKey,
        input: &ApprovePayment,
    ) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
        let (basis, required_balance_accounts, starting_balances) = decision.into_parts();
        let payment = pending_payment(&basis, input.payment)?;
        if payment.initiator() == input.approver {
            return Err(BankProposalDenial::SelfApproval);
        }
        if !basis.is_known_principal(input.approver) {
            return Err(BankProposalDenial::UnknownPrincipal);
        }
        let payload = CanonicalProposalPayload::new("approve-payment")
            .text("payment", &input.payment.canonical_text())
            .u64("approver", input.approver.get());
        let intent = BankIdempotencyClaim::derive(binding, key, payload);
        let mut proposed = basis.clone();
        let journal = append_balanced_transfer(
            &mut proposed,
            payment.source(),
            payment.destination(),
            payment.amount(),
            PostingPurpose::Transfer,
            None,
            intent.key(),
        )?;
        let replacement = payment.with_decision(PaymentStatus::Committed, input.approver);
        proposed.replace_payment(replacement.clone());
        complete_decision_proposal(
            basis,
            required_balance_accounts,
            starting_balances,
            proposed,
            intent,
            vec![
                BankProposedEffect::AppendJournal(journal),
                BankProposedEffect::UpdatePayment {
                    payment: input.payment,
                    replacement,
                },
            ],
        )
    }

    pub fn prepare_reject_payment(
        snapshot: &BankSnapshot,
        binding: BankOperationScopeBinding,
        key: &BankIdempotencyKey,
        input: &RejectPayment,
    ) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
        let payment = pending_payment(snapshot, input.payment)?;
        if payment.initiator() == input.rejecting_principal {
            return Err(BankProposalDenial::SelfApproval);
        }
        if !snapshot.is_known_principal(input.rejecting_principal) {
            return Err(BankProposalDenial::UnknownPrincipal);
        }

        let payload = CanonicalProposalPayload::new("reject-payment")
            .text("payment", &input.payment.canonical_text())
            .u64("rejecting-principal", input.rejecting_principal.get());
        let intent = BankIdempotencyClaim::derive(binding, key, payload);
        let replacement = payment.with_decision(PaymentStatus::Rejected, input.rejecting_principal);
        let mut proposed = snapshot.clone();
        proposed.replace_payment(replacement.clone());
        complete_proposal(
            snapshot,
            proposed,
            intent,
            vec![BankProposedEffect::UpdatePayment {
                payment: input.payment,
                replacement,
            }],
        )
    }

    pub fn prepare_reject_payment_from_decision(
        decision: BankDecisionSnapshot,
        binding: BankOperationScopeBinding,
        key: &BankIdempotencyKey,
        input: &RejectPayment,
    ) -> Result<BankInvariantApprovedProposal, BankProposalDenial> {
        let (basis, required_balance_accounts, starting_balances) = decision.into_parts();
        let payment = pending_payment(&basis, input.payment)?;
        if payment.initiator() == input.rejecting_principal {
            return Err(BankProposalDenial::SelfApproval);
        }
        if !basis.is_known_principal(input.rejecting_principal) {
            return Err(BankProposalDenial::UnknownPrincipal);
        }
        let payload = CanonicalProposalPayload::new("reject-payment")
            .text("payment", &input.payment.canonical_text())
            .u64("rejecting-principal", input.rejecting_principal.get());
        let intent = BankIdempotencyClaim::derive(binding, key, payload);
        let replacement = payment.with_decision(PaymentStatus::Rejected, input.rejecting_principal);
        let mut proposed = basis.clone();
        proposed.replace_payment(replacement.clone());
        complete_decision_proposal(
            basis,
            required_balance_accounts,
            starting_balances,
            proposed,
            intent,
            vec![BankProposedEffect::UpdatePayment {
                payment: input.payment,
                replacement,
            }],
        )
    }
}

fn validate_business_payment(
    snapshot: &BankSnapshot,
    initiator: BankPrincipalId,
    input: &InitiateBusinessPayment,
) -> Result<crate::model::AccountId, BankProposalDenial> {
    if !snapshot.is_known_principal(initiator) {
        return Err(BankProposalDenial::UnknownPrincipal);
    }
    if snapshot.business_account(input.business) != Some(input.from) {
        return Err(BankProposalDenial::AccountOwnershipMismatch);
    }
    let destination = snapshot
        .primary_account(input.recipient)
        .ok_or(BankProposalDenial::UnknownRecipient)?;
    ensure_open(snapshot, input.from)?;
    ensure_open(snapshot, destination)?;
    if destination == input.from {
        return Err(BankProposalDenial::SelfTransfer);
    }
    Ok(destination)
}

fn business_payment(
    idempotency: BankIdempotencyClaim,
    destination: crate::model::AccountId,
    initiator: BankPrincipalId,
    input: &InitiateBusinessPayment,
) -> BusinessPayment {
    BusinessPayment::pending(
        crate::model::PaymentId::from_operation(idempotency.key().bytes(), 0),
        input.business,
        input.from,
        destination,
        initiator,
        input.amount,
    )
}

fn pending_payment(
    snapshot: &BankSnapshot,
    payment_id: crate::model::PaymentId,
) -> Result<&BusinessPayment, BankProposalDenial> {
    let payment = snapshot
        .payment(payment_id)
        .ok_or(BankProposalDenial::UnknownPayment(payment_id))?;
    if payment.status() != PaymentStatus::ApprovalRequired {
        return Err(BankProposalDenial::PaymentAlreadyDecided(payment_id));
    }
    Ok(payment)
}
