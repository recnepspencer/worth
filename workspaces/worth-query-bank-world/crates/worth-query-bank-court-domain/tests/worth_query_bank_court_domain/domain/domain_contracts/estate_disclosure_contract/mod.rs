pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::estate_disclosure_contract::*;

#[test]
fn every_restricted_field_has_domain_owned_classification() {
    assert_eq!(
        RestrictedBankField::CustomerIdentity.classification(),
        BankDisclosureClassification::Restricted
    );
    assert_eq!(
        RestrictedBankField::AccountDetails.classification(),
        BankDisclosureClassification::Restricted
    );
    for field in [
        RestrictedBankField::BeneficiaryIdentity,
        RestrictedBankField::PostingHistory,
        RestrictedBankField::AuditTrail,
        RestrictedBankField::GovernanceMetadata,
        RestrictedBankField::EmergencyAccessActivity,
    ] {
        assert_eq!(
            field.classification(),
            BankDisclosureClassification::HighlyRestricted
        );
    }
    assert_eq!(
        RestrictedBankField::LegalDocument.classification(),
        BankDisclosureClassification::LegalSealed
    );
}

#[test]
fn estate_result_shape_distinguishes_typed_disclosure_from_typed_omission() {
    let estate = EstateCaseId::new(1).unwrap();
    let result = EstateDisclosureResult {
        estate,
        customer: BankDisclosure::Disclosed(BankPrincipalId::new(2).unwrap()),
        beneficiary: BankDisclosure::Omitted(BankDisclosureClassification::HighlyRestricted),
        legal_authority: BankDisclosure::Disclosed(LegalAuthorityId::new(3).unwrap()),
        account: BankDisclosure::Omitted(BankDisclosureClassification::Restricted),
        posting_history: BankDisclosure::Disclosed(EstatePostingHistory {
            postings: [
                EstatePosting {
                    account: AccountId::new(4).unwrap(),
                    amount: SignedMoney::from_minor(-100),
                },
                EstatePosting {
                    account: AccountId::new(5).unwrap(),
                    amount: SignedMoney::from_minor(100),
                },
            ],
        }),
        audit_trail: BankDisclosure::Disclosed(EstateAuditTrail {
            emergency_access: Some(EmergencyAccessId::new(6).unwrap()),
            mandatory_review: MandatoryReviewId::new(7).unwrap(),
        }),
    };

    assert!(matches!(result.customer, BankDisclosure::Disclosed(_)));
    assert_eq!(
        result.beneficiary,
        BankDisclosure::Omitted(BankDisclosureClassification::HighlyRestricted)
    );
    assert_eq!(
        result.account,
        BankDisclosure::<AccountId>::Omitted(BankDisclosureClassification::Restricted)
    );
}

#[test]
fn field_purpose_law_is_an_exact_eight_row_partition() {
    let purposes = [
        EstateCapabilityPurpose::EstateAdministration,
        EstateCapabilityPurpose::IdentityVerification,
        EstateCapabilityPurpose::LegalCompliance,
        EstateCapabilityPurpose::EmergencyProtection,
        EstateCapabilityPurpose::EstateDisbursement,
        EstateCapabilityPurpose::MandatoryReview,
    ];
    let expected = [
        (
            RestrictedBankField::CustomerIdentity,
            [true, true, true, false, false, false],
        ),
        (
            RestrictedBankField::BeneficiaryIdentity,
            [true, false, true, true, false, false],
        ),
        (
            RestrictedBankField::LegalDocument,
            [false, false, true, false, false, false],
        ),
        (
            RestrictedBankField::AccountDetails,
            [true, false, true, true, false, false],
        ),
        (
            RestrictedBankField::PostingHistory,
            [true, false, true, false, false, true],
        ),
        (
            RestrictedBankField::AuditTrail,
            [false, false, true, false, false, true],
        ),
        (
            RestrictedBankField::GovernanceMetadata,
            [true, false, false, false, false, false],
        ),
        (
            RestrictedBankField::EmergencyAccessActivity,
            [false, false, false, true, false, false],
        ),
    ];
    for (field, allowed) in expected {
        assert_eq!(
            purposes.map(|purpose| field.permits(purpose)),
            allowed,
            "disclosure purpose drift for {field:?}"
        );
    }
}
