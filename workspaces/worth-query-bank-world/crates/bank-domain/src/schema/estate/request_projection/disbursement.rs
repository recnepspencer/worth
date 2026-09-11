use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestWorkBudget, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityGovernedInputIdentity,
    ApplicationCapabilityRelatedEntitySelector, ApplicationCapabilityRequest,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
};

use crate::{
    estate::{EstateAction, EstateDisbursement},
    schema::{
        AccountIdentity, BankSchema, CapabilityAccount, DisburseEstateCapability,
        EstateActionContext, EstateCase,
    },
};

use super::estate_request;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.usd-estate-disbursement-governed-input");
const RULE_VERSION: &str = "worth-bank-usd-estate-disbursement-governed-input-v1";

impl ApplicationCapabilityRequest<BankSchema, DisburseEstateCapability> for EstateAction {
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn governed_input_identity(&self) -> Option<ApplicationCapabilityGovernedInputIdentity> {
        let EstateAction::DisburseEstate(disbursement) = *self else {
            return None;
        };
        Some(derive_governed_input("disburse-estate", disbursement))
    }

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::DisburseEstate(disbursement) = *self else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "DisburseEstateOperation",
            ));
        };
        Ok(estate_request(self, disbursement.estate)
            .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
                CapabilityAccount::reference(),
                ApplicationCapabilityEntitySelector::new(
                    AccountIdentity::reference(),
                    crate::schema::encoded_bank_value::<crate::schema::AccountIdBinding>(
                        disbursement.source_account,
                    ),
                ),
            ))
            .magnitude(crate::schema::encoded_bank_value::<
                crate::schema::UsdMoneyBinding,
            >(disbursement.amount)))
    }
}

fn derive_governed_input(
    operation: &'static str,
    disbursement: EstateDisbursement,
) -> ApplicationCapabilityGovernedInputIdentity {
    let [debit, credit] = disbursement.postings;
    let entries = [
        text("operation", operation),
        unsigned("estate", disbursement.estate.get()),
        text(
            "source-account",
            &disbursement.source_account.canonical_text(),
        ),
        text(
            "destination-account",
            &disbursement.destination_account.canonical_text(),
        ),
        unsigned("beneficiary", disbursement.beneficiary.get()),
        signed("amount-minor-units", disbursement.amount.minor_units()),
        text("posting-0-account", &debit.account.canonical_text()),
        signed("posting-0-minor-units", debit.amount.minor_units()),
        text("posting-1-account", &credit.account.canonical_text()),
        signed("posting-1-minor-units", credit.amount.minor_units()),
    ];
    let version = CanonicalizationRuleVersion::new(RULE_VERSION)
        .expect("the fixed estate-disbursement identity rule is valid");
    let prepared = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .expect("estate-disbursement identity loci are unique");
    let budget = CanonicalDigestWorkBudget::new(10, 4_096)
        .expect("the estate-disbursement canonical budget is nonzero");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(prepared, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()
        .expect("bounded estate-disbursement input fits its canonical budget");
    let derived = canonicalization().digest().derive(ready);
    ApplicationCapabilityGovernedInputIdentity::canonical(&derived)
}

fn text(locus: &'static str, value: &str) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::ExactText(value.to_owned().into()),
    )
}

fn unsigned(locus: &'static str, value: u64) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

fn signed(locus: &'static str, value: i64) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: value.into(),
        },
    )
}

fn entry(locus: &'static str, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{
        estate::EstatePosting,
        model::{AccountId, BankPrincipalId, Money, SignedMoney},
    };

    #[test]
    fn identity_covers_variant_and_every_payload_dimension() {
        let basis = [1, 2, 3, 4, 5, 1, -5, 2, 5];
        let mut identities = BTreeSet::new();
        identities.insert(derive_governed_input("other-variant", disbursement(basis)).identity());
        identities.insert(derive_governed_input("disburse-estate", disbursement(basis)).identity());
        for index in 0..basis.len() {
            let mut changed = basis;
            changed[index] = if index == 6 { -9 } else { 9 };
            identities
                .insert(derive_governed_input("disburse-estate", disbursement(changed)).identity());
        }

        assert_eq!(identities.len(), 11);
    }

    #[test]
    fn canonical_identity_reports_one_bounded_ten_entry_digest() {
        let binding = derive_governed_input(
            "disburse-estate",
            disbursement([1, 2, 3, 4, 5, 1, -5, 2, 5]),
        );
        let work = binding.canonical_work().unwrap();

        assert_eq!(work.canonical_entry_count(), 10);
        assert!(work.canonical_encoded_bytes() <= 4_096);
    }

    fn disbursement(parts: [i64; 9]) -> EstateDisbursement {
        let [estate, source, destination, beneficiary, amount, posting_0_account, posting_0_amount, posting_1_account, posting_1_amount] =
            parts;
        EstateDisbursement {
            estate: crate::estate::EstateCaseId::new(estate as u64).unwrap(),
            source_account: AccountId::new(source as u64).unwrap(),
            destination_account: AccountId::new(destination as u64).unwrap(),
            beneficiary: BankPrincipalId::new(beneficiary as u64).unwrap(),
            amount: Money::from_minor(amount).unwrap(),
            postings: [
                EstatePosting {
                    account: AccountId::new(posting_0_account as u64).unwrap(),
                    amount: SignedMoney::from_minor(posting_0_amount),
                },
                EstatePosting {
                    account: AccountId::new(posting_1_account as u64).unwrap(),
                    amount: SignedMoney::from_minor(posting_1_amount),
                },
            ],
        }
    }
}
