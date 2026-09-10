//! Bounded undo intent identity (R8.40 / R8.10).
//!
//! One identity per undo admission. Fan-out of postings, decision facts, or
//! lineage edges must not change the §8 undo_admission counters (1/1/0).

use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;

use super::WorthQueryAftermathDerivationFailure;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.application-aftermath-undo-intent");
const RULE_VERSION: &str = "worth-query-application-aftermath-undo-intent-v1";
const BUDGET: CanonicalDigestWorkBudget = match CanonicalDigestWorkBudget::new(24, 8 * 1_024) {
    Some(budget) => budget,
    None => panic!("fixed undo-intent canonical-work budget is valid"),
};

/// One bounded undo intent identity. Carries original committed and aftermath
/// identities; does not regenerate per posting or lineage edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryUndoIntentIdentity {
    digest: CanonicalDigestId,
    original_product_publication: WorthQueryCommittedProductPublication,
    aftermath_digest: CanonicalDigestId,
    work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryUndoIntentIdentity {
    /// Derive exactly one undo intent identity (R8.40). Touched-record count is
    /// *not* an input — fan-out must not scale this derivation.
    /// Identity inputs only. Posting / decision-fact / lineage fan-out must not
    /// appear here (R8.40). Fan-out twin tests pass those counts as discarded
    /// locals — they never enter the basis.
    pub(crate) fn derive_parts(
        publication: &WorthQueryCommittedProductPublication,
        installed_operation: [u8; 32],
        aftermath_digest: CanonicalDigestId,
        runtime_instance: u64,
    ) -> Result<Self, WorthQueryAftermathDerivationFailure> {
        let version =
            CanonicalizationRuleVersion::new(RULE_VERSION).expect("the undo-intent rule is valid");
        let entries = vec![
            entry(
                "family",
                CanonicalBasisValue::ExactText("undo-intent".into()),
            ),
            entry(
                "runtime-world-owner",
                CanonicalBasisValue::UnsignedInteger {
                    width: CanonicalIntegerWidth::Bits64,
                    value: publication.composite_commit().owner_identity().get().into(),
                },
            ),
            entry(
                "composite-commit",
                CanonicalBasisValue::UnsignedInteger {
                    width: CanonicalIntegerWidth::Bits64,
                    value: publication.composite_commit().ordinal().into(),
                },
            ),
            entry(
                "installed-operation",
                CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(installed_operation)),
            ),
            entry(
                "aftermath",
                CanonicalBasisValue::BytesDigest(aftermath_digest),
            ),
            entry(
                "runtime-instance",
                CanonicalBasisValue::UnsignedInteger {
                    width: CanonicalIntegerWidth::Bits64,
                    value: runtime_instance.into(),
                },
            ),
            // Intentionally omit touched-record / posting / lineage — R8.40.
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
        Ok(Self {
            digest: CanonicalDigestId::new(*derived.value().bytes()),
            original_product_publication: publication.clone(),
            aftermath_digest,
            work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
        })
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub const fn original_product_publication(&self) -> &WorthQueryCommittedProductPublication {
        &self.original_product_publication
    }

    pub const fn aftermath_digest(&self) -> &CanonicalDigestId {
        &self.aftermath_digest
    }

    pub const fn work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.work
    }
}

fn entry(locus: &str, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.to_owned().into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}
