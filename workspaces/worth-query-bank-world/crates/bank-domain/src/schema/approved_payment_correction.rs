//! Compensate the original journal while preserving completed rail settlement.
use super::{
    approved_payment_inbound_binding, ApprovePayment, ApprovePaymentMutationBinding,
    ApprovePaymentOperation, ApprovedPaymentJournal, ApprovedPaymentSettlementEffect, BankSchema,
    InstitutionIdBinding, JournalEntryIdBinding, JournalIdentityField, JournalInstitutionContext,
    ReversalReason, ReverseJournal, ReverseSentMoney, APPROVED_PAYMENT_SETTLEMENT_RAIL,
};
use crate::{
    proposals::BankIdempotencyKey,
    queries::{PaymentDetailQuery, PaymentDetailQueryBinding},
};
use worth_query_decl::facade::{
    application_aftermath::*,
    application_query::ApplicationQueryParameterSet,
    application_schema::{
        ApplicationDeclaredOperation, ApplicationOperationDefinitionBuilder,
        ApplicationReadableScalarValueBinding, ApplicationSchemaDeclarationBuilder,
        WorthQueryExternalEffectCorrelationFamily,
    },
};

pub struct ReverseApprovedPayment;

impl ApplicationDeclaredOperation<BankSchema> for ApprovePaymentOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
        super::decision_read_manifest::install_approved_payment_reads(
            super::program_manifest::install_approved_payment_program(schema),
        )
    }
}
fn journal_field() -> ApplicationCorrectionOutputField<BankSchema> {
    ApplicationCorrectionOutput::<ApprovePaymentMutationBinding, ApprovedPaymentJournal>::field(
        JournalIdentityField::reference(),
    )
}
fn institution_field() -> ApplicationCorrectionOutputField<BankSchema> {
    ApplicationCorrectionOutput::<ApprovePaymentMutationBinding, ApprovedPaymentJournal>::field(
        JournalInstitutionContext::reference(),
    )
}
impl ApplicationCorrectionBinding<BankSchema> for ReverseApprovedPayment {
    const IDENTITY: &'static str = "bank.correction.reverse-approved-payment-journal.v1";
    type Source = ApprovePaymentMutationBinding;
    type InverseIntent = ReverseJournal;
    type Disclosure = PaymentDetailQueryBinding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        BankSchema,
        ApprovePaymentOperation,
        ApprovePayment,
        true,
        false,
    > {
        ApprovePaymentOperation::reference()
            .definition()
            .external_effect_with_inbound(
                ApprovedPaymentSettlementEffect::reference(),
                WorthQueryExternalEffectCorrelationFamily::new(APPROVED_PAYMENT_SETTLEMENT_RAIL)
                    .unwrap(),
                approved_payment_inbound_binding(),
            )
    }
    fn disclosure_parameters(
        _: &ApprovePayment,
    ) -> Result<
        ApplicationQueryParameterSet<PaymentDetailQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<BankSchema> {
        DeclaredCorrectionMechanism::compensation_binding::<Self>(
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: "bank.approved-payment-journal-compensated.v1".to_owned(),
            },
        )
        .expect("the completed payment's journal has an explicit business compensation")
    }
    fn completed_external_reconciliation() -> Option<DeclaredReconciliationProcedure> {
        Some(DeclaredReconciliationProcedure::new("confirm-approved-payment-settlement").unwrap())
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<BankSchema>> {
        vec![journal_field(), institution_field()]
    }
    fn derive_correction(
        _: &ApprovePayment,
        material: ApplicationCorrectionSourceMaterial<'_, BankSchema>,
    ) -> Result<ReverseJournal, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning;
        let ApplicationCorrectionSourceMaterial::Compensation(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let journal = JournalEntryIdBinding::decode(
            fields
                .field(&journal_field())
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?;
        let institution = InstitutionIdBinding::decode(
            fields
                .field(&institution_field())
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?;
        Ok(ReverseJournal {
            institution,
            journal,
            reason: ReversalReason::OperatorCorrection,
        })
    }
    fn inverse_key(
        key: &str,
    ) -> Result<
        BankIdempotencyKey,
        worth_query_decl::facade::application_schema::ApplicationValueEncodeDenial,
    > {
        ReverseSentMoney::inverse_key(key)
    }
    fn visit<V: ApplicationCorrectionVisitor<BankSchema>>(
        dispatch: ApplicationCorrectionDispatch<BankSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
