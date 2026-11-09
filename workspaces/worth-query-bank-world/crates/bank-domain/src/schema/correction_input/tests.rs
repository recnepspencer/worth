//! Canonical DATA recovery exercises real correction declaration factories.
use super::*;
use crate::schema::{
    BankSchema, RestoreRecordedEstateFreeze, ReverseApprovedPayment, ReverseSentMoney,
};
use crate::{
    estate::{DeathNoticeId, EstateCaseId},
    model::{AccountId, BankPrincipalId, PaymentId},
};
use worth_query_decl::facade::application_aftermath::ApplicationCorrectionBinding;
fn bounds() -> ValueDecodeBounds {
    ValueDecodeBounds::new(4096, 256, 64, 16)
}
fn roundtrip<C: ApplicationCorrectionBinding<BankSchema>>(
    input: <C::Source as ApplicationMutationBinding<BankSchema>>::Input,
) where
    <C::Source as ApplicationMutationBinding<BankSchema>>::Input: std::fmt::Debug + PartialEq,
{
    let encoded = ApplicationEncodedInput::<
        <C::Source as ApplicationMutationBinding<BankSchema>>::InputBinding,
    >::encode(input)
    .unwrap();
    let capture = encoded.capture_for_retention().unwrap();
    let restored = C::recover_source_input(
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
fn send_money_correction_factory_keeps_fixture_and_operation_created_accounts() {
    for from in [
        AccountId::new(1).unwrap(),
        AccountId::from_operation([7; 32], 3),
    ] {
        roundtrip::<ReverseSentMoney>(SendMoney {
            from,
            recipient: BankPrincipalId::new(2).unwrap(),
            amount: Money::from_minor(37).unwrap(),
        });
    }
}
#[test]
fn payment_correction_factory_keeps_the_original_payment_identity() {
    for payment in [
        PaymentId::new(1).unwrap(),
        PaymentId::from_operation([9; 32], 4),
    ] {
        roundtrip::<ReverseApprovedPayment>(ApprovePayment {
            payment,
            approver: BankPrincipalId::new(2).unwrap(),
        });
    }
}
#[test]
fn estate_correction_factory_keeps_the_original_freeze_subjects() {
    roundtrip::<RestoreRecordedEstateFreeze>(EstateAction::FreezeAccount {
        estate: EstateCaseId::new(1).unwrap(),
        account: AccountId::from_operation([8; 32], 5),
    });
}
struct WireBinding;
impl ApplicationStructuredValueBinding for WireBinding {
    type Value = RecordedSendMoney;
    const IDENTITY_NAME: &'static str = SendMoneyInputBinding::IDENTITY_NAME;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
#[test]
fn correction_factory_refuses_recorded_nonpositive_money_and_zero_domain_identities() {
    for (account, principal, amount) in [(1, 2, 0), (1, 2, -1), (0, 2, 3), (1, 0, 3)] {
        let value = RecordedSendMoney {
            from: RecordedAccountId(RecordedCreatedId::Fixture(account)),
            recipient: RecordedPrincipalId(principal),
            amount: RecordedMoney {
                minor_units: amount,
                _currency: PhantomData,
            },
        };
        // Intentionally malformed DATA under the actual binding identity. No
        // operation, World identity, authority or retained receipt is fabricated.
        let encoded = ApplicationEncodedInput::<WireBinding>::encode(value).unwrap();
        let capture = encoded.capture_for_retention().unwrap();
        assert!(matches!(
            ReverseSentMoney::recover_source_input(
                capture.recorded_bytes(),
                CanonicalDigestId::new(*encoded.identity()),
                bounds()
            ),
            Err(ApplicationRetainedInputDenial::Codec(
                ValueCodecDenial::Malformed
            ))
        ));
    }
}
#[test]
fn correction_factory_refuses_a_foreign_source_binding_and_an_unadmitted_estate_action() {
    let encoded = ApplicationEncodedInput::<SendMoneyInputBinding>::encode(SendMoney {
        from: AccountId::new(1).unwrap(),
        recipient: BankPrincipalId::new(2).unwrap(),
        amount: Money::from_minor(3).unwrap(),
    })
    .unwrap();
    let capture = encoded.capture_for_retention().unwrap();
    assert!(matches!(
        ReverseApprovedPayment::recover_source_input(
            capture.recorded_bytes(),
            CanonicalDigestId::new(*encoded.identity()),
            bounds()
        ),
        Err(ApplicationRetainedInputDenial::BindingMismatch)
    ));
    let value = EstateAction::NotifyDeath {
        estate: EstateCaseId::new(1).unwrap(),
        notice: DeathNoticeId::new(2).unwrap(),
        subject: BankPrincipalId::new(3).unwrap(),
    };
    let encoded = ApplicationEncodedInput::<EstateActionInputBinding>::encode(value).unwrap();
    let capture = encoded.capture_for_retention().unwrap();
    assert!(matches!(
        RestoreRecordedEstateFreeze::recover_source_input(
            capture.recorded_bytes(),
            CanonicalDigestId::new(*encoded.identity()),
            bounds()
        ),
        Err(ApplicationRetainedInputDenial::Codec(
            ValueCodecDenial::Malformed
        ))
    ));
}
