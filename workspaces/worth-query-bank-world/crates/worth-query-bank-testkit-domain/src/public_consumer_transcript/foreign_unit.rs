//! A hostile binding retains the value codec while omitting its currency unit.
use super::*;
use worth_query_decl::facade::application_schema::{fragment::*, *};
type OriginalSet = <PaymentRecords as Fragment>::Bindings;
type OriginalMoney = <OriginalSet as Scalar<SignedMoney<USD>>>::Binding;
type PostingRow = bank_domain::schema::PostingValueValue;
type Sequence = bank_domain::model::AccountJournalRevision;
type PurposeValue = bank_domain::schema::PostingPurpose;
pub struct ForeignFields;
impl Fragment for ForeignFields {
    const NAME: &'static str = <PaymentRecords as Fragment>::NAME;
    type Bindings = ForeignBindings;
    type Members = (Posting,);
}
pub struct ForeignBindings;
impl BindingSet for ForeignBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        w: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(w)
    }
}
impl Scalar<SignedMoney<USD>> for ForeignBindings {
    type Binding = UnitlessMoney;
    const WRITABLE: bool = <OriginalSet as Scalar<SignedMoney<USD>>>::WRITABLE;
}
impl Scalar<Sequence> for ForeignBindings {
    type Binding = <OriginalSet as Scalar<Sequence>>::Binding;
    const WRITABLE: bool = <OriginalSet as Scalar<Sequence>>::WRITABLE;
}
impl Scalar<PurposeValue> for ForeignBindings {
    type Binding = <OriginalSet as Scalar<PurposeValue>>::Binding;
    const WRITABLE: bool = <OriginalSet as Scalar<PurposeValue>>::WRITABLE;
}
impl Record<PostingRow> for ForeignBindings {
    fn record() -> impl Shape<PostingRow, Self> {
        worth_query_decl::facade::application_schema::fragment::field(
            "PostingAmount",
            |row: &PostingRow| row.posting_amount,
        )
        .and("PostingAccountSequence", |row: &PostingRow| {
            row.posting_account_sequence
        })
        .equality_queryable(true)
        .and("Purpose", |row: &PostingRow| row.purpose)
        .equality_queryable(true)
        .build(
            |posting_amount,
             posting_account_sequence,
             purpose,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PostingRow {
                    posting_amount,
                    posting_account_sequence,
                    purpose,
                })
            },
        )
    }
}
pub(super) fn field() -> FieldSelection<ForeignFields, Posting, PostingValue, SignedMoney<USD>> {
    FieldSelection::at("PostingAmount")
}
pub struct UnitlessMoney;
impl ApplicationScalarValueBinding for UnitlessMoney {
    type Value = SignedMoney<USD>;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <OriginalMoney as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = OriginalMoney::IDENTITY_NAME;
    const SCALAR_FAMILY: worth_query_decl::facade::typed::ScalarAspectType =
        OriginalMoney::SCALAR_FAMILY;
    const FRAME: Option<ApplicationFrameIdentity> = OriginalMoney::FRAME;
    const SUPPORTS_ADMITTED_DECODE: bool = OriginalMoney::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        OriginalMoney::NATIVE_INPUT_ADMISSION;
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        OriginalMoney::canonical_contract()
    }
    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        OriginalMoney::validate(value)
    }
    fn encode(value: &Self::Value) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        OriginalMoney::encode(value)
    }
}
impl ApplicationReadableScalarValueBinding for UnitlessMoney {
    fn decode(value: &ApplicationValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        OriginalMoney::decode(value)
    }
    fn decode_admitted(
        value: &ApplicationValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        OriginalMoney::decode_admitted(value, work)
    }
}
