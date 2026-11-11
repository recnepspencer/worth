//! Factories restore recorded corrective inputs without invoking domain derivation.
use super::*;
use crate::{
    estate::EstateCaseId,
    model::{AccountId, BankPrincipalId, InstitutionId, JournalEntryId, Money},
    schema::{
        BankSchema, RestoreRecordedEstateFreeze, ReverseApprovedPayment, ReverseSentMoney,
        SendMoney, SendMoneyInputBinding,
    },
};
use worth_query_decl::facade::application_aftermath::{
    ApplicationCorrectionBinding, ApplicationInverseBinding,
};
fn bounds() -> ValueDecodeBounds {
    ValueDecodeBounds::new(4096, 256, 64, 16)
}
fn roundtrip<C: ApplicationCorrectionBinding<BankSchema>>(
    value: <ApplicationInverseBinding<BankSchema, C> as ApplicationMutationBinding<BankSchema>>::Input,
) where
    <ApplicationInverseBinding<BankSchema, C> as ApplicationMutationBinding<BankSchema>>::Input:
        std::fmt::Debug + PartialEq,
{
    let encoded = ApplicationEncodedInput::<<ApplicationInverseBinding<BankSchema,C> as ApplicationMutationBinding<BankSchema>>::InputBinding>::encode(value).unwrap();
    let capture = encoded.capture_for_retention().unwrap();
    let restored = C::recover_inverse_input(
        capture.recorded_bytes(),
        CanonicalDigestId::new(*encoded.identity()),
        bounds(),
    )
    .unwrap();
    assert_eq!(restored.input(), encoded.input());
    assert_eq!(restored.identity(), encoded.identity());
    assert_eq!(
        restored.capture_for_retention().unwrap().recorded_bytes(),
        capture.recorded_bytes()
    );
}
#[test]
fn journal_inverse_input_factories_keep_original_identities_and_declared_reason() {
    for journal in [
        JournalEntryId::new(1).unwrap(),
        JournalEntryId::from_operation([0x34; 32], 5),
    ] {
        for reason in [
            ReversalReason::Duplicate,
            ReversalReason::OperatorCorrection,
            ReversalReason::ExternalReturn,
        ] {
            let input = ReverseJournal {
                institution: InstitutionId::new(2).unwrap(),
                journal,
                reason,
            };
            roundtrip::<ReverseSentMoney>(input.clone());
            roundtrip::<ReverseApprovedPayment>(input);
        }
    }
}
#[test]
fn estate_inverse_input_factory_preserves_declared_prior_status_data() {
    // DATA reconstruction preserves declared values; fresh operation admission
    // still decides whether any restored value is legal to execute.
    for prior_status in [
        AccountStatus::Open,
        AccountStatus::Frozen,
        AccountStatus::Closed,
    ] {
        roundtrip::<RestoreRecordedEstateFreeze>(RestoreFrozenEstateAccount {
            estate: EstateCaseId::new(1).unwrap(),
            account: AccountId::from_operation([0x56; 32], 7),
            prior_status,
        });
    }
}
struct WireBinding;
impl ApplicationStructuredValueBinding for WireBinding {
    type Value = RecordedReverseJournal;
    const IDENTITY_NAME: &'static str = ReverseJournalInputBinding::IDENTITY_NAME;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
#[test]
fn inverse_input_factory_refuses_foreign_binding_and_invalid_domain_identities() {
    let source = ApplicationEncodedInput::<ReverseJournalInputBinding>::encode(ReverseJournal {
        institution: InstitutionId::new(1).unwrap(),
        journal: JournalEntryId::new(2).unwrap(),
        reason: ReversalReason::OperatorCorrection,
    })
    .unwrap();
    let capture = source.capture_for_retention().unwrap();
    assert!(matches!(
        ReverseSentMoney::recover_source_input(
            capture.recorded_bytes(),
            CanonicalDigestId::new(*source.identity()),
            bounds()
        ),
        Err(ApplicationRetainedInputDenial::BindingMismatch)
    ));
    let source = ApplicationEncodedInput::<SendMoneyInputBinding>::encode(SendMoney {
        from: AccountId::new(1).unwrap(),
        recipient: BankPrincipalId::new(2).unwrap(),
        amount: Money::from_minor(3).unwrap(),
    })
    .unwrap();
    let capture = source.capture_for_retention().unwrap();
    assert!(matches!(
        ReverseSentMoney::recover_inverse_input(
            capture.recorded_bytes(),
            CanonicalDigestId::new(*source.identity()),
            bounds()
        ),
        Err(ApplicationRetainedInputDenial::BindingMismatch)
    ));
    for (institution, journal) in [(0, 1), (1, 0)] {
        // Malformed DATA only; no operation or historical authority is fabricated.
        let value = ApplicationEncodedInput::<WireBinding>::encode(RecordedReverseJournal {
            institution: RecordedInstitutionId(institution),
            journal: RecordedJournalId(RecordedCreatedId::Fixture(journal)),
            reason: RecordedReason::OperatorCorrection,
        })
        .unwrap();
        let capture = value.capture_for_retention().unwrap();
        assert!(matches!(
            ReverseSentMoney::recover_inverse_input(
                capture.recorded_bytes(),
                CanonicalDigestId::new(*value.identity()),
                bounds()
            ),
            Err(ApplicationRetainedInputDenial::Codec(
                ValueCodecDenial::Malformed
            ))
        ));
    }
}
