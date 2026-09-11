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
    estate::EstateAction,
    schema::{
        AccountIdentity, BankSchema, CapabilityAccount, EstateActionContext, EstateCase,
        FreezeEstateAccountCapability,
    },
};

use super::estate_request;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.freeze-estate-account-governed-input");
const RULE_VERSION: &str = "worth-bank-freeze-estate-account-governed-input-v1";

impl ApplicationCapabilityRequest<BankSchema, FreezeEstateAccountCapability> for EstateAction {
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn governed_input_identity(&self) -> Option<ApplicationCapabilityGovernedInputIdentity> {
        let EstateAction::FreezeAccount { estate, account } = *self else {
            return None;
        };
        let entries = [
            unsigned("estate", estate.get()),
            text("account", &account.canonical_text()),
        ];
        Some(canonical_identity(entries))
    }

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::FreezeAccount { estate, account } = *self else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "FreezeEstateAccountOperation",
            ));
        };
        Ok(estate_request(self, estate).related_entity(
            ApplicationCapabilityRelatedEntitySelector::new(
                CapabilityAccount::reference(),
                ApplicationCapabilityEntitySelector::new(
                    AccountIdentity::reference(),
                    crate::schema::encoded_bank_value::<crate::schema::AccountIdBinding>(account),
                ),
            ),
        ))
    }
}

fn canonical_identity(
    entries: [CanonicalBasisEntry; 2],
) -> ApplicationCapabilityGovernedInputIdentity {
    let version = CanonicalizationRuleVersion::new(RULE_VERSION)
        .expect("the fixed freeze-input identity rule is valid");
    let prepared = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .expect("freeze-input identity loci are unique");
    let budget = CanonicalDigestWorkBudget::new(2, 1_024)
        .expect("the freeze-input canonical budget is nonzero");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(prepared, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()
        .expect("bounded freeze input fits its canonical budget");
    ApplicationCapabilityGovernedInputIdentity::canonical(
        &canonicalization().digest().derive(ready),
    )
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
    use super::*;
    use crate::{estate::EstateCaseId, model::AccountId};

    #[test]
    fn governed_identity_binds_both_estate_and_account() {
        let original = identity(action(1, 7));
        assert_ne!(original, identity(action(2, 7)));
        assert_ne!(original, identity(action(1, 8)));
    }

    fn identity(action: EstateAction) -> ApplicationCapabilityGovernedInputIdentity {
        <EstateAction as ApplicationCapabilityRequest<
            BankSchema,
            FreezeEstateAccountCapability,
        >>::governed_input_identity(&action)
        .unwrap()
    }

    fn action(estate: u64, account: u64) -> EstateAction {
        EstateAction::FreezeAccount {
            estate: EstateCaseId::new(estate).unwrap(),
            account: AccountId::new(account).unwrap(),
        }
    }
}
