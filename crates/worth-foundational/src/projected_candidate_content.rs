//! Versioned descriptive content of a complete projected candidate universe.
/// Canonical commitment to full-generation occurrences, projected slots/values,
/// physical support keys, ordering/cardinality and every candidate disposition.
/// Content is interpreted under its exact installed projection/Parent namespace;
/// the digest alone is not a self-contained projection contract.
/// This is representation only: it grants no equality, READ, currentness,
/// correction eligibility, or publication permission. Consumers must independently
/// compare genuine captured bodies and preserve their installed authority binding.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProjectedCandidateContentV1([u8; 32]);
impl ProjectedCandidateContentV1 {
    pub const DOMAIN: &'static [u8] = b"worth:projected-candidate-content:v1";
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}
