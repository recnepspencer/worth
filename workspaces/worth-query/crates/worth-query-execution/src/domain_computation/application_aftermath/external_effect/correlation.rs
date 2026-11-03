//! Typed Query correlation identity for external-effect recovery (R8.5).
//!
//! Provider-supplied strings may appear only in diagnostics and must never
//! participate in equality that admits a posture transition.

use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily;

use super::super::WorthQueryAftermathDerivationFailure;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.external-effect-correlation");
const RULE_VERSION: &str = "worth-query-external-effect-correlation-v1";
const BUDGET: CanonicalDigestWorkBudget = match CanonicalDigestWorkBudget::new(24, 8 * 1_024) {
    Some(budget) => budget,
    None => panic!("fixed external-effect correlation budget is valid"),
};

/// Query-owned correlation identity. Equality is digest equality only.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExternalEffectCorrelationIdentity {
    digest: CanonicalDigestId,
}

impl ExternalEffectCorrelationIdentity {
    pub(crate) const fn from_digest(digest: CanonicalDigestId) -> Self {
        Self { digest }
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub fn bytes(&self) -> &[u8; 32] {
        self.digest.bytes()
    }
}

/// Inputs that may bind correlation. Provider strings are excluded by type.
pub struct ExternalEffectCorrelationBasis<'a> {
    pub correlation_family: WorthQueryExternalEffectCorrelationFamily,
    pub operation_slot: &'a str,
    pub operation_version: u64,
    pub outcome_identity: u64,
    pub idempotency_key: &'a [u8; 32],
    pub branch: &'a str,
}

pub fn derive_external_effect_correlation_identity(
    basis: ExternalEffectCorrelationBasis<'_>,
) -> Result<ExternalEffectCorrelationIdentity, WorthQueryAftermathDerivationFailure> {
    if basis.operation_slot.trim().is_empty() || basis.branch.trim().is_empty() {
        return Err(WorthQueryAftermathDerivationFailure::EmptyCorrelationBasis);
    }
    let version = CanonicalizationRuleVersion::new(RULE_VERSION)
        .expect("the external-effect correlation rule is valid");
    let entries = vec![
        entry(
            "family",
            CanonicalBasisValue::ExactText("external-effect-correlation".into()),
        ),
        entry(
            "correlation-family",
            CanonicalBasisValue::ExactText(basis.correlation_family.as_str().to_owned().into()),
        ),
        entry(
            "operation-slot",
            CanonicalBasisValue::ExactText(basis.operation_slot.to_owned().into()),
        ),
        entry(
            "operation-version",
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: basis.operation_version.into(),
            },
        ),
        entry(
            "outcome-identity",
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: basis.outcome_identity.into(),
            },
        ),
        entry(
            "idempotency-key",
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(*basis.idempotency_key)),
        ),
        entry(
            "branch",
            CanonicalBasisValue::ExactText(basis.branch.to_owned().into()),
        ),
    ];
    let prepared = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .map_err(|_| WorthQueryAftermathDerivationFailure::BasisRejected)?;
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(prepared, CanonicalDigestAlgorithmId::sha256(), BUDGET)
        .into_result()
        .map_err(|_| WorthQueryAftermathDerivationFailure::DigestRejected)?;
    let derived = canonicalization().digest().derive(ready);
    Ok(ExternalEffectCorrelationIdentity {
        digest: CanonicalDigestId::new(*derived.value().bytes()),
    })
}

fn entry(locus: &str, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.to_owned().into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}
