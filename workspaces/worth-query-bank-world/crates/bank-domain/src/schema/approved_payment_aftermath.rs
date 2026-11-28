//! Installation of the approved payment's rail and its aftermath contract.
use super::{
    approved_payment_inbound_binding, ApprovePayment, ApprovePaymentOperation,
    ApprovedPaymentSettlementEffect, BankSchema, ReverseJournalMutationBinding,
    APPROVED_PAYMENT_SETTLEMENT_RAIL,
};
use worth_query_decl::facade::{
    application_aftermath::{
        DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract, DeclaredCompensation,
        DeclaredCorrectionMechanism, DeclaredReconciliationProcedure,
    },
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationDeclaredOperation, ApplicationOperationDefinition,
        ApplicationSchemaDeclarationBuilder, WorthQueryExternalEffectCorrelationFamily,
    },
};

pub(super) fn approved_payment_definition(
) -> ApplicationOperationDefinition<BankSchema, ApprovePaymentOperation, ApprovePayment> {
    ApprovePaymentOperation::reference()
        .definition()
        .external_effect_with_inbound(
            ApprovedPaymentSettlementEffect::reference(),
            WorthQueryExternalEffectCorrelationFamily::new(APPROVED_PAYMENT_SETTLEMENT_RAIL)
                .expect("Bank's settlement rail has a valid family"),
            approved_payment_inbound_binding(),
        )
        .aftermath(
            DeclaredApplicationAftermathContract::runtime_with_external_owner(
                DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new(
                    ReverseJournalMutationBinding::descriptor().operation_name(),
                    DeclaredAftermathPostcondition::BusinessPostcondition {
                        identity: "bank.approved-payment-journal-compensated.v1".into(),
                    },
                )),
                DeclaredReconciliationProcedure::new("confirm-approved-payment-settlement"),
            ),
        )
        .finish()
}

impl ApplicationDeclaredOperation<BankSchema> for ApprovePaymentOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
        super::decision_read_manifest::install_approved_payment_reads(
            super::program_manifest::install_approved_payment_program(schema),
        )
    }
}
