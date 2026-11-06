//! Payment rows of the bank model.
use super::*;
pub struct PaymentIdentity;
impl fragment::Aspect for PaymentIdentity {
    const NAME: &'static str = "PaymentIdentity";
    type Row = PaymentIdentityValue;
}
impl PaymentIdentity {
    pub fn reference() -> AspectSelection<PaymentRecords, PaymentIntent, Self> {
        aspect(PaymentRecords, PaymentIntent, Self)
    }
}
pub struct PaymentIdentityValue {
    pub payment_identity_field: PaymentId,
}
impl fragment::Record<PaymentIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<PaymentIdentityValue, Self> {
        fragment::field("PaymentIdentityField", |row: &PaymentIdentityValue| {
            row.payment_identity_field
        })
        .build(|payment_identity_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PaymentIdentityValue {
                payment_identity_field,
            })
        })
    }
}
pub struct PostingValue;
impl fragment::Aspect for PostingValue {
    const NAME: &'static str = "PostingValue";
    type Row = PostingValueValue;
}
impl PostingValue {
    pub fn reference() -> AspectSelection<PaymentRecords, Posting, Self> {
        aspect(PaymentRecords, Posting, Self)
    }
}
pub struct PostingValueValue {
    pub posting_amount: SignedMoney<USD>,
    pub posting_account_sequence: AccountJournalRevision,
    pub purpose: PostingPurpose,
}
impl fragment::Record<PostingValueValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<PostingValueValue, Self> {
        fragment::field("PostingAmount", |row: &PostingValueValue| {
            row.posting_amount
        })
        .and("PostingAccountSequence", |row: &PostingValueValue| {
            row.posting_account_sequence
        })
        .and("Purpose", |row: &PostingValueValue| row.purpose)
        .build(
            |posting_amount,
             posting_account_sequence,
             purpose,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PostingValueValue {
                    posting_amount,
                    posting_account_sequence,
                    purpose,
                })
            },
        )
    }
}
pub struct PostingIdentity;
impl fragment::Aspect for PostingIdentity {
    const NAME: &'static str = "PostingIdentity";
    type Row = PostingIdentityValue;
}
impl PostingIdentity {
    pub fn reference() -> AspectSelection<PaymentRecords, Posting, Self> {
        aspect(PaymentRecords, Posting, Self)
    }
}
pub struct PostingIdentityValue {
    pub posting_identity_field: PostingId,
}
impl fragment::Record<PostingIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<PostingIdentityValue, Self> {
        fragment::field("PostingIdentityField", |row: &PostingIdentityValue| {
            row.posting_identity_field
        })
        .build(|posting_identity_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PostingIdentityValue {
                posting_identity_field,
            })
        })
    }
}
pub struct JournalIdentity;
impl fragment::Aspect for JournalIdentity {
    const NAME: &'static str = "JournalIdentity";
    type Row = JournalIdentityValue;
}
impl JournalIdentity {
    pub fn reference() -> AspectSelection<PaymentRecords, JournalEntry, Self> {
        aspect(PaymentRecords, JournalEntry, Self)
    }
}
pub struct JournalIdentityValue {
    pub journal_identity_field: JournalEntryId,
    pub journal_institution_context: InstitutionId,
}
impl fragment::Record<JournalIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<JournalIdentityValue, Self> {
        fragment::field("JournalIdentityField", |row: &JournalIdentityValue| {
            row.journal_identity_field
        })
        .and("JournalInstitutionContext", |row: &JournalIdentityValue| {
            row.journal_institution_context
        })
        .build(
            |journal_identity_field,
             journal_institution_context,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(JournalIdentityValue {
                    journal_identity_field,
                    journal_institution_context,
                })
            },
        )
    }
}
pub struct JournalState;
impl fragment::Aspect for JournalState {
    const NAME: &'static str = "JournalState";
    type Row = JournalStateValue;
}
impl JournalState {
    pub fn reference() -> AspectSelection<PaymentRecords, JournalEntry, Self> {
        aspect(PaymentRecords, JournalEntry, Self)
    }
}
pub struct JournalStateValue {
    pub journal_purpose: PostingPurpose,
}
impl fragment::Record<JournalStateValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<JournalStateValue, Self> {
        fragment::field("JournalPurpose", |row: &JournalStateValue| {
            row.journal_purpose
        })
        .build(|journal_purpose, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(JournalStateValue { journal_purpose })
        })
    }
}
pub struct PaymentState;
impl fragment::Aspect for PaymentState {
    const NAME: &'static str = "PaymentState";
    type Row = PaymentStateValue;
}
impl PaymentState {
    pub fn reference() -> AspectSelection<PaymentRecords, PaymentIntent, Self> {
        aspect(PaymentRecords, PaymentIntent, Self)
    }
}
pub struct PaymentStateValue {
    pub payment_status_field: PaymentStatus,
}
impl fragment::Record<PaymentStateValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<PaymentStateValue, Self> {
        fragment::field("PaymentStatusField", |row: &PaymentStateValue| {
            row.payment_status_field
        })
        .build(|payment_status_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PaymentStateValue {
                payment_status_field,
            })
        })
    }
}
pub struct PaymentValue;
impl fragment::Aspect for PaymentValue {
    const NAME: &'static str = "PaymentValue";
    type Row = PaymentValueValue;
}
impl PaymentValue {
    pub fn reference() -> AspectSelection<PaymentRecords, PaymentIntent, Self> {
        aspect(PaymentRecords, PaymentIntent, Self)
    }
}
pub struct PaymentValueValue {
    pub payment_amount: Money<USD>,
}
impl fragment::Record<PaymentValueValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<PaymentValueValue, Self> {
        fragment::field("PaymentAmount", |row: &PaymentValueValue| {
            row.payment_amount
        })
        .build(|payment_amount, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PaymentValueValue { payment_amount })
        })
    }
}
