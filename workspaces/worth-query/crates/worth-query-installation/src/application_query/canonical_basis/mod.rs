mod continuation;
mod graph;
mod installed_query;
mod planning;

use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalizationRuleVersion,
};

pub(in crate::application_query) use continuation::{
    prepare_continuation_basis, ContinuationCanonicalInput,
};
pub(in crate::application_query) use graph::prepare_graph_basis;
pub(in crate::application_query) use installed_query::prepare_installed_query_basis;
pub(in crate::application_query) use planning::prepare_planning_basis;

const DOMAIN_NAME: &str = "worth-query.application-query-installation";
const RULE_VERSION: &str = "worth-query-application-query-installation-v2";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCanonicalArtifact {
    basis: CanonicalBasisReadyArtifact,
    digest: CanonicalDigestId,
    work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryApplicationCanonicalArtifact {
    pub fn basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.basis
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub const fn work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.work
    }
}

fn prepare_artifact(
    family: &'static str,
    mut entries: Vec<CanonicalBasisEntry>,
    budget: CanonicalDigestWorkBudget,
) -> Result<WorthQueryApplicationCanonicalArtifact, CanonicalDigestDerivationDenial> {
    entries.insert(0, text("family", family));
    let version = CanonicalizationRuleVersion::new(RULE_VERSION)
        .expect("the fixed installed application-query rule is valid");
    let basis = prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Future(DOMAIN_NAME),
        entries,
    )
    .into_result()
    .expect("installed application-query meaning always has a nonempty basis");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis.clone(), CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()?;
    let derived = canonicalization().digest().derive(ready);
    Ok(WorthQueryApplicationCanonicalArtifact {
        basis,
        digest: CanonicalDigestId::new(*derived.value().bytes()),
        work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    })
}

fn text(locus: impl Into<String>, value: impl Into<String>) -> CanonicalBasisEntry {
    entry(locus, CanonicalBasisValue::ExactText(value.into().into()))
}

fn unsigned(locus: impl Into<String>, value: usize) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::facade::CanonicalIntegerWidth::Bits64,
            value: u64::try_from(value)
                .expect("application-query structural counts fit in u64")
                .into(),
        },
    )
}

fn digest(locus: impl Into<String>, value: &CanonicalDigestId) -> CanonicalBasisEntry {
    entry(locus, CanonicalBasisValue::BytesDigest(*value))
}

fn entry(locus: impl Into<String>, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Future(DOMAIN_NAME),
        worth_foundational::facade::CanonicalBasisLocus::Named(locus.into().into()),
        worth_foundational::facade::CanonicalBasisEntryKind::Field,
        value,
    )
}

fn append_managed_index(
    entries: &mut Vec<CanonicalBasisEntry>,
    selection: Option<
        &worth_query_declaration::facade::application_query::ApplicationQueryManagedIndexMeaning,
    >,
) {
    if let Some(selection) = selection {
        let parts = selection.parts();
        entries.extend([
            unsigned("managed-index.version", usize::from(parts.semantic_version)),
            text("managed-index.index", &parts.index),
            text("managed-index.probe-parameter", &parts.probe_parameter),
        ]);
    }
}
