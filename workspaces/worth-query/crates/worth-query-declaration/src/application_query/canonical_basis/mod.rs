mod definition;
mod entry;
mod result_shape;

use std::cmp::Ordering;

use worth_foundational::facade::{
    prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact,
    CanonicalizationRuleVersion, InternedString,
};

pub(in crate::application_query) use definition::prepare_definition_basis;

pub const APPLICATION_QUERY_DOMAIN: &str = "worth-query.application-query";
const APPLICATION_QUERY_RULE: &str = "worth-query-application-query-v3";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationQueryCanonicalArtifact {
    basis: CanonicalBasisReadyArtifact,
}

impl ApplicationQueryCanonicalArtifact {
    pub fn basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.basis
    }

    pub fn embedded_entries(
        &self,
        domain: CanonicalBasisDomain,
        locus_prefix: &str,
        kind: CanonicalBasisEntryKind,
    ) -> Vec<CanonicalBasisEntry> {
        self.basis
            .payload()
            .entries()
            .iter()
            .map(|entry| {
                let name = match entry.locus() {
                    CanonicalBasisLocus::Named(InternedString::Raw(name)) => name,
                    _ => unreachable!(
                        "application-query canonical construction creates only named raw loci"
                    ),
                };
                CanonicalBasisEntry::new(
                    domain,
                    CanonicalBasisLocus::Named(format!("{locus_prefix}.{name}").into()),
                    kind,
                    entry.value().clone(),
                )
            })
            .collect()
    }
}

impl Ord for ApplicationQueryCanonicalArtifact {
    fn cmp(&self, other: &Self) -> Ordering {
        self.basis
            .payload()
            .entries()
            .cmp(other.basis.payload().entries())
    }
}

impl PartialOrd for ApplicationQueryCanonicalArtifact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub(super) fn prepare_artifact(
    entries: Vec<CanonicalBasisEntry>,
) -> ApplicationQueryCanonicalArtifact {
    let version = CanonicalizationRuleVersion::new(APPLICATION_QUERY_RULE)
        .expect("the fixed application-query canonicalization rule is valid");
    let basis = prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Future(APPLICATION_QUERY_DOMAIN),
        entries,
    )
    .into_result()
    .expect("application-query meaning always has a nonempty canonical basis");
    ApplicationQueryCanonicalArtifact { basis }
}
