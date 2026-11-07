use std::cmp::Ordering;

use worth_foundational::facade::{
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, InternedString,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSchemaIdentity(CanonicalBasisReadyArtifact);

impl ApplicationSchemaIdentity {
    pub(super) const fn from_canonical_basis(value: CanonicalBasisReadyArtifact) -> Self {
        Self(value)
    }

    pub fn canonical_basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.0
    }

    pub fn embedded_entries(
        &self,
        domain: CanonicalBasisDomain,
        locus_prefix: &str,
        kind: CanonicalBasisEntryKind,
    ) -> Vec<CanonicalBasisEntry> {
        self.0
            .payload()
            .entries()
            .iter()
            .map(|entry| {
                let name = match entry.locus() {
                    CanonicalBasisLocus::Named(InternedString::Stored(name)) => name.as_str(),
                    CanonicalBasisLocus::Named(InternedString::Raw(name)) => name.as_str(),
                    _ => unreachable!(
                        "application-schema canonical construction creates only named raw loci"
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

impl Ord for ApplicationSchemaIdentity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.payload().entries().cmp(other.0.payload().entries())
    }
}

impl PartialOrd for ApplicationSchemaIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
