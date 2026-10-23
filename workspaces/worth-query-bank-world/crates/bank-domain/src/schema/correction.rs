//! SendMoney compensation derives the original journal under current disclosure.
use worth_query_decl::facade::{
    application_aftermath::*,
    application_query::ApplicationQueryParameterSet,
    application_schema::{
        ApplicationOperationDefinitionBuilder, ApplicationReadableScalarValueBinding,
    },
};

use super::{
    BankSchema, InstitutionIdBinding, JournalEntryIdBinding, JournalIdentityField,
    JournalInstitutionContext, ReversalReason, ReverseJournal, SendMoney, SendMoneyJournal,
    SendMoneyMutationBinding, SendMoneyOperation,
};
use crate::{
    proposals::BankIdempotencyKey,
    queries::{AccountSummaryQuery, AccountSummaryQueryBinding},
};

pub struct ReverseSentMoney;

fn journal_field() -> ApplicationCorrectionOutputField<BankSchema> {
    ApplicationCorrectionOutput::<SendMoneyMutationBinding, SendMoneyJournal>::field(
        JournalIdentityField::reference(),
    )
}

fn institution_field() -> ApplicationCorrectionOutputField<BankSchema> {
    ApplicationCorrectionOutput::<SendMoneyMutationBinding, SendMoneyJournal>::field(
        JournalInstitutionContext::reference(),
    )
}

impl ApplicationCorrectionBinding<BankSchema> for ReverseSentMoney {
    const IDENTITY: &'static str = "bank.correction.reverse-sent-money.v1";
    type Source = SendMoneyMutationBinding;
    type InverseIntent = ReverseJournal;
    type Disclosure = AccountSummaryQueryBinding;

    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<BankSchema, SendMoneyOperation, SendMoney, true, false>
    {
        SendMoneyOperation::reference()
            .definition()
            .no_external_effect()
    }

    fn disclosure_parameters(
        _: &SendMoney,
    ) -> Result<
        ApplicationQueryParameterSet<AccountSummaryQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }

    fn mechanism() -> DeclaredCorrectionMechanism<BankSchema> {
        DeclaredCorrectionMechanism::compensation_binding::<Self>(
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: "bank.original-journal-balanced-reversal.v1".to_owned(),
            },
        )
        .expect("Bank compensation declares a named business postcondition")
    }

    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<BankSchema>> {
        vec![journal_field(), institution_field()]
    }

    fn derive_correction(
        _: &SendMoney,
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
        use worth_query_decl::facade::{
            application_schema::{ApplicationValueEncodeDenial, ApplicationValueValidationDenial},
            portable_identity::WorthQueryPortableTypeIdentity,
        };
        BankIdempotencyKey::new(key).map_err(|_| {
            ApplicationValueEncodeDenial::Validation(ApplicationValueValidationDenial::rejected(
                WorthQueryPortableTypeIdentity::declared("bank.application-mutation-client-key.v1"),
                "expected a Bank idempotency key",
            ))
        })
    }

    fn visit<V: ApplicationCorrectionVisitor<BankSchema>>(
        dispatch: ApplicationCorrectionDispatch<BankSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
