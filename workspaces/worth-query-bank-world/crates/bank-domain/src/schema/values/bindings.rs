use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
use worth_query_decl::facade::{
    application_schema::{
        ApplicationIdentityScalarValueBinding, ApplicationReadableScalarValueBinding,
        ApplicationScalarValueBinding, ApplicationSignedAggregateValueBinding,
        ApplicationUnitIdentity, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
        ApplicationValueEncodeDenial, ApplicationValueIsIdentity, ApplicationValueIsNotIdentity,
        ApplicationValueSignedAggregateAvailable, ApplicationValueSignedAggregateUnavailable,
        ApplicationValueValidationDenial,
    },
    worth_query_value_binding,
};

use crate::model::{
    AccountAuthorizationId, AccountId, AccountJournalRevision, AccountName, BankPrincipalId,
    BusinessId, CustomerRole, EmployeeAssignmentId, EmployeeRole, InstitutionId, JournalEntryId,
    Money, PaymentId, PostingId, SignedMoney, USD,
};
use crate::proposals::{BankIdempotencyIntent, BankIdempotencyKeyIdentity};

use super::{AccountKind, AccountStatus, PaymentStatus, PostingPurpose};

pub(crate) fn encoded_bank_value<Binding>(
    value: Binding::Value,
) -> worth_query_decl::facade::application_schema::ApplicationEncodedScalarValue<Binding>
where
    Binding: ApplicationScalarValueBinding,
{
    worth_query_decl::facade::application_schema::ApplicationEncodedScalarValue::<Binding>::try_new(
        value,
    )
    .expect("bank declaration values are valid for their exact binding")
}

trait StringCodec: Sized {
    fn encode(&self) -> InternedString;
    fn decode(value: InternedString) -> Option<Self>;
}

fn encode_string<T: StringCodec>(value: &T) -> InternedString {
    value.encode()
}

fn decode_string<T: StringCodec>(value: InternedString) -> Option<T> {
    T::decode(value)
}

macro_rules! string_enum_binding {
    ($Binding:ident for $Value:ty, $identity:literal, {$($Variant:path => $text:literal),+ $(,)?}) => {
        impl StringCodec for $Value {
            fn encode(&self) -> InternedString {
                match self { $($Variant => InternedString::from($text)),+ }
            }

            fn decode(value: InternedString) -> Option<Self> {
                let InternedString::Raw(value) = value else { return None; };
                match value.as_str() { $($text => Some($Variant),)+ _ => None }
            }
        }

        worth_query_value_binding! {
            pub $Binding for $Value {
                identity: $identity,
                scalar: String,
                encode: encode_string,
                decode: decode_string,
            }
        }
    };
}

string_enum_binding!(AccountKindBinding for AccountKind, "bank.account-kind.v1", {
    AccountKind::Personal => "personal",
    AccountKind::Business => "business",
    AccountKind::InstitutionCash => "institution-cash",
    AccountKind::InstitutionSettlement => "institution-settlement",
});
string_enum_binding!(AccountStatusBinding for AccountStatus, "bank.account-status.v1", {
    AccountStatus::Open => "open",
    AccountStatus::Frozen => "frozen",
    AccountStatus::Closed => "closed",
});
string_enum_binding!(PaymentStatusBinding for PaymentStatus, "bank.payment-status.v1", {
    PaymentStatus::Pending => "pending",
    PaymentStatus::ApprovalRequired => "approval-required",
    PaymentStatus::Committed => "committed",
    PaymentStatus::Rejected => "rejected",
    PaymentStatus::Reversed => "reversed",
});
string_enum_binding!(PostingPurposeBinding for PostingPurpose, "bank.posting-purpose.v1", {
    PostingPurpose::OpeningFunding => "opening-funding",
    PostingPurpose::Deposit => "deposit",
    PostingPurpose::Withdrawal => "withdrawal",
    PostingPurpose::Transfer => "transfer",
    PostingPurpose::EstateDisbursement => "estate-disbursement",
    PostingPurpose::Reversal => "reversal",
});
string_enum_binding!(CustomerRoleBinding for CustomerRole, "bank.customer-role.v1", {
    CustomerRole::PersonalOwner => "personal-owner",
    CustomerRole::BusinessOwner => "business-owner",
    CustomerRole::Initiator => "initiator",
    CustomerRole::Approver => "approver",
    CustomerRole::Viewer => "viewer",
});
string_enum_binding!(EmployeeRoleBinding for EmployeeRole, "bank.employee-role.v1", {
    EmployeeRole::Teller => "teller",
    EmployeeRole::Auditor => "auditor",
    EmployeeRole::BranchManager => "branch-manager",
    EmployeeRole::EstateSpecialist => "estate-specialist",
    EmployeeRole::Compliance => "compliance",
    EmployeeRole::Legal => "legal",
});

macro_rules! unsigned_identity_binding {
    ($Binding:ident for $Value:ty, $identity:literal, $decode:path) => {
        worth_query_value_binding! {
            pub $Binding for $Value {
                identity: $identity,
                scalar: UInt64,
                encode: encode_unsigned_identity,
                decode: $decode,
            }
        }
    };
}

trait UnsignedIdentity {
    fn get(&self) -> u64;
}

fn encode_unsigned_identity<T: UnsignedIdentity>(value: &T) -> u64 {
    value.get()
}

macro_rules! unsigned_identity {
    ($($Value:ty),+ $(,)?) => {
        $(impl UnsignedIdentity for $Value {
            fn get(&self) -> u64 { <$Value>::get(*self) }
        })+
    };
}

unsigned_identity!(BusinessId, EmployeeAssignmentId, InstitutionId);

unsigned_identity_binding!(BusinessIdBinding for BusinessId, "bank.business-id.v1", BusinessId::new);
unsigned_identity_binding!(EmployeeAssignmentIdBinding for EmployeeAssignmentId, "bank.employee-assignment-id.v1", EmployeeAssignmentId::new);
unsigned_identity_binding!(InstitutionIdBinding for InstitutionId, "bank.institution-id.v1", InstitutionId::new);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPrincipalIdBinding;

impl ApplicationScalarValueBinding for BankPrincipalIdBinding {
    type Value = BankPrincipalId;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

    const IDENTITY_NAME: &'static str = "bank.principal-id.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::UInt64;

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }

    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::UInt64(value.get()))
    }
}

impl ApplicationReadableScalarValueBinding for BankPrincipalIdBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        let AspectValue::UInt64(value) = value else {
            return Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY.clone(),
                expected: Self::SCALAR_FAMILY,
                observed: value.value_family(),
            });
        };
        BankPrincipalId::new(*value).ok_or_else(|| ApplicationValueDecodeDenial::CodecRejected {
            binding_identity: Self::IDENTITY.clone(),
        })
    }
}

impl ApplicationIdentityScalarValueBinding for BankPrincipalIdBinding {}

macro_rules! canonical_text_binding {
    ($Binding:ident for $Value:ty, $identity:literal, $decode:path) => {
        impl StringCodec for $Value {
            fn encode(&self) -> InternedString {
                InternedString::from(self.canonical_text())
            }
            fn decode(value: InternedString) -> Option<Self> {
                let InternedString::Raw(value) = value else {
                    return None;
                };
                $decode(&value)
            }
        }

        worth_query_value_binding! {
            pub $Binding for $Value {
                identity: $identity,
                scalar: String,
                encode: encode_string,
                decode: decode_string,
            }
        }
    };
}

canonical_text_binding!(AccountIdBinding for AccountId, "bank.account-id.v1", AccountId::parse_canonical_text);
canonical_text_binding!(AccountAuthorizationIdBinding for AccountAuthorizationId, "bank.account-authorization-id.v1", AccountAuthorizationId::parse_canonical_text);
canonical_text_binding!(PaymentIdBinding for PaymentId, "bank.payment-id.v1", PaymentId::parse_canonical_text);
canonical_text_binding!(JournalEntryIdBinding for JournalEntryId, "bank.journal-entry-id.v1", JournalEntryId::parse_canonical_text);
canonical_text_binding!(PostingIdBinding for PostingId, "bank.posting-id.v1", PostingId::parse_canonical_text);

fn encode_account_journal_revision(value: &AccountJournalRevision) -> u64 {
    value.get()
}

fn decode_account_journal_revision(value: u64) -> Option<AccountJournalRevision> {
    Some(AccountJournalRevision::from_posting_count(value))
}

worth_query_value_binding! {
    pub AccountJournalRevisionBinding for AccountJournalRevision {
        identity: "bank.account-journal-revision.v1",
        scalar: UInt64,
        encode: encode_account_journal_revision,
        decode: decode_account_journal_revision,
    }
}

fn encode_account_name(value: &AccountName) -> InternedString {
    InternedString::from(value.as_str())
}

fn decode_account_name(value: InternedString) -> Option<AccountName> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    AccountName::new(value).ok()
}

worth_query_value_binding! {
    pub AccountNameBinding for AccountName {
        identity: "bank.account-name.v1",
        scalar: String,
        encode: encode_account_name,
        decode: decode_account_name,
    }
}

macro_rules! usd_money_binding {
    ($Binding:ident for $Value:ty, $identity:literal, $decode:path, $signed:ty) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $Binding;

        impl ApplicationScalarValueBinding for $Binding {
            type Value = $Value;
            type Unit = USD;
            type Decode = ApplicationValueDecodeAvailable;
            type Identity = ApplicationValueIsNotIdentity;
            type SignedAggregate = $signed;

            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::Int64;
            const UNIT: Option<ApplicationUnitIdentity> =
                Some(ApplicationUnitIdentity::declared("UsdCurrency"));

            fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
                Ok(())
            }

            fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
                Ok(AspectValue::Int64(value.minor_units()))
            }
        }

        impl ApplicationReadableScalarValueBinding for $Binding {
            fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
                let AspectValue::Int64(value) = value else {
                    return Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                        binding_identity: Self::IDENTITY.clone(),
                        expected: Self::SCALAR_FAMILY,
                        observed: value.value_family(),
                    });
                };
                ($decode)(*value).ok_or_else(|| ApplicationValueDecodeDenial::CodecRejected {
                    binding_identity: Self::IDENTITY.clone(),
                })
            }
        }
    };
}

fn decode_usd_money(value: i64) -> Option<Money<USD>> {
    Money::from_minor(value).ok()
}

fn decode_signed_usd_money(value: i64) -> Option<SignedMoney<USD>> {
    Some(SignedMoney::from_minor(value))
}

usd_money_binding!(UsdMoneyBinding for Money<USD>, "bank.usd-money.v1", decode_usd_money, ApplicationValueSignedAggregateUnavailable);
usd_money_binding!(SignedUsdMoneyBinding for SignedMoney<USD>, "bank.signed-usd-money.v1", decode_signed_usd_money, ApplicationValueSignedAggregateAvailable);

impl ApplicationSignedAggregateValueBinding for SignedUsdMoneyBinding {
    fn decode_aggregate(value: i64) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        Ok(SignedMoney::from_minor(value))
    }
}

canonical_text_binding!(BankIdempotencyKeyIdentityBinding for BankIdempotencyKeyIdentity, "bank.idempotency-key-identity.v1", BankIdempotencyKeyIdentity::from_canonical_text);
canonical_text_binding!(BankIdempotencyIntentBinding for BankIdempotencyIntent, "bank.idempotency-intent.v1", BankIdempotencyIntent::from_canonical_text);
