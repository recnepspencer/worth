use worth_query_decl::facade::{
    application_aftermath::{
        DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract, DeclaredCompensation,
        DeclaredCorrectionMechanism, DeclaredReconciliationProcedure,
    },
    application_schema::{
        ApplicationOperationDefinition, ApplicationOperationRef,
        ApplicationSchemaDeclarationBuilder, WorthQueryExternalEffectCorrelationFamily,
    },
    worth_query_application_contribution,
};

use crate::authorization::{
    install_payment_ability_policies, ApproveBusinessFunds, InitiateBusinessFunds,
    SendPersonalFunds, ViewPayment,
};

use super::super::{
    decision_read_manifest::install_payment_decision_reads,
    entities::*,
    fields::*,
    governance::{AccountActivityEffect, DistinctApproverPolicy, UsdCurrency},
    install_approved_business_payment_authority, install_approved_payment_assessment,
    install_payment_approval_grant_minting,
    operations::*,
    posting_integrity::posting_integrity_invariant,
    precondition_manifest::install_payment_preconditions,
    program_manifest::{install_money_programs, install_payment_program},
    relations::*,
    ApprovedBusinessPaymentAdvanceBinding, ApprovedBusinessPaymentAdvanceOperation,
    ApprovedBusinessPaymentApprovalBinding, ApprovedBusinessPaymentApprovalOperation,
    ApprovedBusinessPaymentAuthoringBinding, ApprovedBusinessPaymentAuthoringOperation,
    ApprovedBusinessPaymentInstanceStartBinding, ApprovedBusinessPaymentInstanceStartOperation,
    ApprovedPaymentSettlementEffect, BankSchema, APPROVED_PAYMENT_SETTLEMENT_RAIL,
};

worth_query_application_contribution! {
    pub contribution BankPayments in BankSchema {
        identity: "worth.bank.payments.v1",
        members: |schema| {
            let schema = install_payment_members(schema)
                .ability(ViewPayment::reference())
                .ability(SendPersonalFunds::reference())
                .ability(InitiateBusinessFunds::reference())
                .ability(ApproveBusinessFunds::reference())
                .operation(without_external_effect_or_aftermath(
                    ApplyOpeningFundingOperation::reference(),
                ))
                .application_mutation_binding::<ApplyOpeningFundingMutationBinding>()
                .operation(without_external_effect_or_aftermath(DepositOperation::reference()))
                .application_mutation_binding::<DepositMutationBinding>()
                .operation(without_external_effect_or_aftermath(WithdrawOperation::reference()))
                .application_mutation_binding::<WithdrawMutationBinding>()
                .operation(without_external_effect_or_aftermath(SendMoneyOperation::reference()))
                .application_mutation_binding::<SendMoneyMutationBinding>()
                .operation(without_external_effect_or_aftermath(
                    InitiateBusinessPaymentOperation::reference(),
                ))
                .operation(without_external_effect_or_aftermath(
                    ApprovedBusinessPaymentAuthoringOperation::reference(),
                ))
                .application_mutation_binding::<ApprovedBusinessPaymentAuthoringBinding>()
                .operation(without_external_effect_or_aftermath(
                    ApprovedBusinessPaymentApprovalOperation::reference(),
                ))
                .application_mutation_binding::<ApprovedBusinessPaymentApprovalBinding>()
                .operation(without_external_effect_or_aftermath(
                    ApprovedBusinessPaymentInstanceStartOperation::reference(),
                ))
                .application_mutation_binding::<ApprovedBusinessPaymentInstanceStartBinding>()
                .operation(without_external_effect_or_aftermath(
                    ApprovedBusinessPaymentAdvanceOperation::reference(),
                ))
                .application_mutation_binding::<ApprovedBusinessPaymentAdvanceBinding>()
                .operation(approved_payment_operation())
                .operation(without_external_effect_or_aftermath(
                    RejectPaymentOperation::reference(),
                ))
                .operation(without_external_effect_or_aftermath(
                    ReverseJournalOperation::reference(),
                ))
                .application_mutation_binding::<ReverseJournalMutationBinding>();
            let schema = install_approved_business_payment_authority(schema);
            let schema = install_approved_payment_assessment(schema);
            let schema = install_money_programs(schema);
            let schema = install_payment_program(schema);
            let schema = install_payment_approval_grant_minting(schema);
            let schema = install_payment_preconditions(schema);
            let schema = install_payment_decision_reads(schema)
                .invariant(posting_integrity_invariant())
                .policy(DistinctApproverPolicy::reference())
                .unit(UsdCurrency::reference())
                .effect(AccountActivityEffect::reference())
                .effect(ApprovedPaymentSettlementEffect::reference())
                .application_query(crate::queries::payment_detail_definition())
                .application_mutation_binding::<InitiateBusinessPaymentMutationBinding>()
                .application_mutation_binding::<ApprovePaymentMutationBinding>()
                .application_mutation_binding::<RejectPaymentMutationBinding>()
                .application_query_binding::<crate::queries::PaymentDetailQueryBinding>()
                .application_query(crate::queries::pending_payments_definition())
                .application_query_binding::<crate::queries::PendingPaymentsQueryBinding>();
            install_payment_operation_abilities(install_payment_ability_policies(schema))
        }
    }
}

fn install_payment_members(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .entity(PaymentIntent::reference())
        .entity(Approval::reference())
        .entity(JournalEntry::reference())
        .entity(Posting::reference())
        .aspect(PaymentIntent::reference(), PaymentIdentity::reference())
        .aspect(Posting::reference(), PostingValue::reference())
        .aspect(Posting::reference(), PostingIdentity::reference())
        .aspect(JournalEntry::reference(), JournalIdentity::reference())
        .aspect(JournalEntry::reference(), JournalState::reference())
        .aspect(PaymentIntent::reference(), PaymentState::reference())
        .aspect(PaymentIntent::reference(), PaymentValue::reference())
        .field(
            PaymentIntent::reference(),
            PaymentIdentityField::reference(),
        )
        .field(Posting::reference(), PostingAmount::reference())
        .field(Posting::reference(), PostingAccountSequence::reference())
        .field(Posting::reference(), PostingIdentityField::reference())
        .field(Posting::reference(), Purpose::reference())
        .field(JournalEntry::reference(), JournalIdentityField::reference())
        .field(JournalEntry::reference(), JournalPurpose::reference())
        .field(PaymentIntent::reference(), PaymentStatusField::reference())
        .field(PaymentIntent::reference(), PaymentAmount::reference())
        .relation(
            PaymentSource::reference(),
            PaymentIntent::reference(),
            Account::reference(),
        )
        .relation(
            PaymentDestination::reference(),
            PaymentIntent::reference(),
            Account::reference(),
        )
        .relation(
            PaymentBusiness::reference(),
            PaymentIntent::reference(),
            Business::reference(),
        )
        .relation(
            PaymentInitiator::reference(),
            Principal::reference(),
            PaymentIntent::reference(),
        )
        .relation(
            PaymentApproval::reference(),
            PaymentIntent::reference(),
            Approval::reference(),
        )
        .relation(
            ApprovalPrincipal::reference(),
            Approval::reference(),
            Principal::reference(),
        )
        .relation(
            JournalPosting::reference(),
            JournalEntry::reference(),
            Posting::reference(),
        )
        .relation(
            JournalReversal::reference(),
            JournalEntry::reference(),
            JournalEntry::reference(),
        )
        .relation(
            PostingAccount::reference(),
            Posting::reference(),
            Account::reference(),
        )
}

fn install_payment_operation_abilities(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    use crate::authorization::ServiceInstitutionAccount;

    schema
        .operation_requires_ability(
            ApplyOpeningFundingOperation::reference(),
            ServiceInstitutionAccount::reference(),
        )
        .operation_requires_ability(
            DepositOperation::reference(),
            ServiceInstitutionAccount::reference(),
        )
        .operation_requires_ability(
            WithdrawOperation::reference(),
            ServiceInstitutionAccount::reference(),
        )
        .operation_requires_ability(
            SendMoneyOperation::reference(),
            SendPersonalFunds::reference(),
        )
        .operation_requires_ability(
            InitiateBusinessPaymentOperation::reference(),
            InitiateBusinessFunds::reference(),
        )
        .operation_requires_ability(
            ApprovePaymentOperation::reference(),
            ApproveBusinessFunds::reference(),
        )
        .operation_requires_ability(
            RejectPaymentOperation::reference(),
            ApproveBusinessFunds::reference(),
        )
        .operation_requires_ability(
            ReverseJournalOperation::reference(),
            ServiceInstitutionAccount::reference(),
        )
}

fn without_external_effect_or_aftermath<Operation, Input>(
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
) -> ApplicationOperationDefinition<BankSchema, Operation, Input> {
    operation
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
}

fn approved_payment_operation(
) -> ApplicationOperationDefinition<BankSchema, ApprovePaymentOperation, ApprovePayment> {
    ApprovePaymentOperation::reference()
        .definition()
        .external_effect(
            ApprovedPaymentSettlementEffect::reference(),
            WorthQueryExternalEffectCorrelationFamily::new(APPROVED_PAYMENT_SETTLEMENT_RAIL)
                .expect("the approved-payment settlement rail is an atomic identity"),
        )
        .aftermath(
            DeclaredApplicationAftermathContract::runtime_with_external_owner(
                DeclaredCorrectionMechanism::Compensation(
                    DeclaredCompensation::new(
                        "reverse-approved-payment-journal",
                        DeclaredAftermathPostcondition::BusinessPostcondition {
                            identity: "approved-payment-settlement-corrected".into(),
                        },
                    )
                    .expect("approved-payment compensation is well-formed"),
                ),
                DeclaredReconciliationProcedure::new("confirm-approved-payment-settlement")
                    .expect("approved-payment reconciliation is well-formed"),
            ),
        )
        .finish()
}
