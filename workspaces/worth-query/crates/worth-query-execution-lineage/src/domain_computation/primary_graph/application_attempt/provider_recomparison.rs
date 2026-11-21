//! Provider precondition recomparison evidence.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationPreconditionComparisonEvidence {
    expected_version_count: usize,
    expected_fact_count: usize,
    identity: Option<[u8; 32]>,
}

impl WorthQueryMutationPreconditionComparisonEvidence {
    pub const fn from_parts(
        expected_version_count: usize,
        expected_fact_count: usize,
        identity: Option<[u8; 32]>,
    ) -> Self {
        Self {
            expected_version_count,
            expected_fact_count,
            identity,
        }
    }

    pub const fn expected_version_count(&self) -> usize {
        self.expected_version_count
    }

    pub const fn expected_fact_count(&self) -> usize {
        self.expected_fact_count
    }

    pub const fn identity(&self) -> Option<&[u8; 32]> {
        self.identity.as_ref()
    }
}
