mod bindings;

pub use bindings::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountKind {
    Personal,
    Business,
    InstitutionCash,
    InstitutionSettlement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum AccountStatus {
    Open,
    Frozen,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum PaymentStatus {
    Pending,
    ApprovalRequired,
    Committed,
    Rejected,
    Reversed,
}

impl PaymentStatus {
    /// Stable text for identity derivation. These bytes are part of every
    /// derived assessment identity, so they never follow a rename.
    pub const fn canonical_text(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::ApprovalRequired => "ApprovalRequired",
            Self::Committed => "Committed",
            Self::Rejected => "Rejected",
            Self::Reversed => "Reversed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostingPurpose {
    OpeningFunding,
    Deposit,
    Withdrawal,
    Transfer,
    EstateDisbursement,
    Reversal,
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{AspectValue, InternedString, StoredValueText, Symbol};
    use worth_query_decl::facade::application_schema::{
        ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    };

    use super::{
        AccountAuthorizationIdBinding, AccountIdBinding, AccountKind, AccountKindBinding,
        AccountNameBinding, AccountStatus, AccountStatusBinding, BankIdempotencyIntentBinding,
        BankIdempotencyKeyIdentityBinding, CustomerRoleBinding, EmployeeRoleBinding,
        JournalEntryIdBinding, PaymentIdBinding, PaymentStatus, PaymentStatusBinding,
        PostingIdBinding, PostingPurpose, PostingPurposeBinding,
    };
    use crate::model::{
        AccountAuthorizationId, AccountId, AccountName, CustomerRole, EmployeeRole, JournalEntryId,
        PaymentId, PostingId,
    };
    use crate::proposals::BankIdempotencyClaim;

    fn assert_canonical_carriers<Binding>(expected: Binding::Value, text: &str)
    where
        Binding: ApplicationReadableScalarValueBinding,
        Binding::Value: std::fmt::Debug + PartialEq,
    {
        assert_eq!(
            Binding::encode(&expected).unwrap(),
            AspectValue::String(InternedString::Raw(text.to_owned()))
        );
        for carrier in [
            InternedString::Raw(text.to_owned()),
            InternedString::Stored(StoredValueText::new(text.to_owned())),
        ] {
            assert_eq!(
                Binding::decode(&AspectValue::String(carrier)).unwrap(),
                expected
            );
        }
    }

    fn assert_unknown_enum_tag_refused<Binding: ApplicationReadableScalarValueBinding>() {
        assert!(Binding::decode(&AspectValue::String(InternedString::Symbol(Symbol(7)))).is_err());
        for text in ["", "unknown-tag"] {
            for carrier in [
                InternedString::Raw(text.to_owned()),
                InternedString::Stored(StoredValueText::new(text.to_owned())),
            ] {
                assert!(Binding::decode(&AspectValue::String(carrier)).is_err());
            }
        }
    }

    #[test]
    fn account_kind_and_status_bindings_decode_raw_and_stored_tags() {
        for (value, tag) in [
            (AccountKind::Personal, "personal"),
            (AccountKind::Business, "business"),
            (AccountKind::InstitutionCash, "institution-cash"),
            (AccountKind::InstitutionSettlement, "institution-settlement"),
        ] {
            assert_canonical_carriers::<AccountKindBinding>(value, tag);
        }
        for (value, tag) in [
            (AccountStatus::Open, "open"),
            (AccountStatus::Frozen, "frozen"),
            (AccountStatus::Closed, "closed"),
        ] {
            assert_canonical_carriers::<AccountStatusBinding>(value, tag);
        }
    }

    #[test]
    fn other_enum_bindings_decode_raw_and_stored_tags() {
        for (value, tag) in [
            (PaymentStatus::Pending, "pending"),
            (PaymentStatus::ApprovalRequired, "approval-required"),
            (PaymentStatus::Committed, "committed"),
            (PaymentStatus::Rejected, "rejected"),
            (PaymentStatus::Reversed, "reversed"),
        ] {
            assert_canonical_carriers::<PaymentStatusBinding>(value, tag);
        }
        for (value, tag) in [
            (PostingPurpose::OpeningFunding, "opening-funding"),
            (PostingPurpose::Deposit, "deposit"),
            (PostingPurpose::Withdrawal, "withdrawal"),
            (PostingPurpose::Transfer, "transfer"),
            (PostingPurpose::EstateDisbursement, "estate-disbursement"),
            (PostingPurpose::Reversal, "reversal"),
        ] {
            assert_canonical_carriers::<PostingPurposeBinding>(value, tag);
        }
        for (value, tag) in [
            (CustomerRole::PersonalOwner, "personal-owner"),
            (CustomerRole::BusinessOwner, "business-owner"),
            (CustomerRole::Initiator, "initiator"),
            (CustomerRole::Approver, "approver"),
            (CustomerRole::Viewer, "viewer"),
        ] {
            assert_canonical_carriers::<CustomerRoleBinding>(value, tag);
        }
        for (value, tag) in [
            (EmployeeRole::Teller, "teller"),
            (EmployeeRole::Auditor, "auditor"),
            (EmployeeRole::BranchManager, "branch-manager"),
            (EmployeeRole::EstateSpecialist, "estate-specialist"),
            (EmployeeRole::Compliance, "compliance"),
            (EmployeeRole::Legal, "legal"),
        ] {
            assert_canonical_carriers::<EmployeeRoleBinding>(value, tag);
        }
    }

    #[test]
    fn enum_bindings_refuse_unknown_tags_and_symbols() {
        assert_unknown_enum_tag_refused::<AccountKindBinding>();
        assert_unknown_enum_tag_refused::<AccountStatusBinding>();
        assert_unknown_enum_tag_refused::<PaymentStatusBinding>();
        assert_unknown_enum_tag_refused::<PostingPurposeBinding>();
        assert_unknown_enum_tag_refused::<CustomerRoleBinding>();
        assert_unknown_enum_tag_refused::<EmployeeRoleBinding>();
    }

    #[test]
    fn account_id_binding_decodes_raw_and_stored_fixture_identity() {
        assert_canonical_carriers::<AccountIdBinding>(AccountId::new(10).unwrap(), "fixture:10");
    }

    #[test]
    fn account_id_binding_refuses_symbols_and_invalid_canonical_text() {
        assert!(
            AccountIdBinding::decode(&AspectValue::String(InternedString::Symbol(Symbol(7))))
                .is_err()
        );
        for text in ["", "fixture:0", "fixture:invalid", "unrecognized"] {
            for carrier in [
                InternedString::Raw(text.to_owned()),
                InternedString::Stored(StoredValueText::new(text.to_owned())),
            ] {
                assert!(AccountIdBinding::decode(&AspectValue::String(carrier)).is_err());
            }
        }
    }

    #[test]
    fn other_canonical_bindings_round_trip_raw_and_stored_text() {
        assert_canonical_carriers::<AccountAuthorizationIdBinding>(
            AccountAuthorizationId::new(11).unwrap(),
            "fixture:11",
        );
        assert_canonical_carriers::<PaymentIdBinding>(PaymentId::new(12).unwrap(), "fixture:12");
        assert_canonical_carriers::<JournalEntryIdBinding>(
            JournalEntryId::new(13).unwrap(),
            "fixture:13",
        );
        assert_canonical_carriers::<PostingIdBinding>(PostingId::new(14).unwrap(), "fixture:14");
        let claim = BankIdempotencyClaim::from_application_binding([0x12; 32], [0xab; 32]);
        assert_canonical_carriers::<BankIdempotencyKeyIdentityBinding>(
            claim.key(),
            &"12".repeat(32),
        );
        assert_canonical_carriers::<BankIdempotencyIntentBinding>(claim.intent(), &"ab".repeat(32));
    }

    #[test]
    fn account_name_binding_decodes_raw_and_stored_text() {
        let text = "Primary account";
        let expected = AccountName::new(text).unwrap();
        for carrier in [
            InternedString::Raw(text.to_owned()),
            InternedString::Stored(StoredValueText::new(text.to_owned())),
        ] {
            let decoded = AccountNameBinding::decode(&AspectValue::String(carrier)).unwrap();
            assert_eq!(decoded, expected);
            assert_eq!(decoded.as_str(), text);
        }
    }

    #[test]
    fn account_name_binding_refuses_symbols_and_invalid_text() {
        assert!(
            AccountNameBinding::decode(&AspectValue::String(InternedString::Symbol(Symbol(7))))
                .is_err()
        );
        for text in ["", " untrimmed", "control\ntext"] {
            for carrier in [
                InternedString::Raw(text.to_owned()),
                InternedString::Stored(StoredValueText::new(text.to_owned())),
            ] {
                assert!(AccountNameBinding::decode(&AspectValue::String(carrier)).is_err());
            }
        }
    }

    #[test]
    fn payment_status_canonical_text_is_pinned() {
        let pinned = [
            (PaymentStatus::Pending, "Pending"),
            (PaymentStatus::ApprovalRequired, "ApprovalRequired"),
            (PaymentStatus::Committed, "Committed"),
            (PaymentStatus::Rejected, "Rejected"),
            (PaymentStatus::Reversed, "Reversed"),
        ];
        for (status, text) in pinned {
            assert_eq!(status.canonical_text(), text);
        }
    }

    #[test]
    fn estate_disbursement_is_a_distinct_round_tripping_posting_purpose() {
        let encoded = PostingPurposeBinding::encode(&PostingPurpose::EstateDisbursement).unwrap();

        assert_eq!(
            PostingPurposeBinding::decode(&encoded).unwrap(),
            PostingPurpose::EstateDisbursement
        );
        assert_ne!(
            encoded,
            PostingPurposeBinding::encode(&PostingPurpose::Transfer).unwrap(),
            "estate disbursement must not masquerade as an ordinary transfer"
        );
    }
}
