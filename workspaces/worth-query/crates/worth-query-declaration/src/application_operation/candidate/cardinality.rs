use super::ApplicationCandidateResourceCeiling;

/// Fixed-shape mutation cardinality admitted before candidate allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCandidateCardinalityCeiling {
    maximum_creates: usize,
    maximum_deletes: usize,
    maximum_links: usize,
    maximum_unlinks: usize,
    maximum_writes: usize,
    maximum_emits: usize,
}

impl ApplicationCandidateCardinalityCeiling {
    pub const fn fixed(
        maximum_creates: usize,
        maximum_deletes: usize,
        maximum_links: usize,
        maximum_unlinks: usize,
        maximum_writes: usize,
        maximum_emits: usize,
    ) -> Self {
        Self {
            maximum_creates,
            maximum_deletes,
            maximum_links,
            maximum_unlinks,
            maximum_writes,
            maximum_emits,
        }
    }

    pub const fn maximum_creates(self) -> usize {
        self.maximum_creates
    }

    pub const fn maximum_deletes(self) -> usize {
        self.maximum_deletes
    }

    pub const fn maximum_links(self) -> usize {
        self.maximum_links
    }

    pub const fn maximum_unlinks(self) -> usize {
        self.maximum_unlinks
    }

    pub const fn maximum_writes(self) -> usize {
        self.maximum_writes
    }

    pub const fn maximum_emits(self) -> usize {
        self.maximum_emits
    }
}

/// Complete declaration-time candidate reservation contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCandidateRequirements {
    cardinality: ApplicationCandidateCardinalityCeiling,
    resources: ApplicationCandidateResourceCeiling,
}

impl ApplicationCandidateRequirements {
    pub const fn fixed_shape(
        cardinality: ApplicationCandidateCardinalityCeiling,
        resources: ApplicationCandidateResourceCeiling,
    ) -> Self {
        Self {
            cardinality,
            resources,
        }
    }

    pub const fn cardinality(self) -> ApplicationCandidateCardinalityCeiling {
        self.cardinality
    }

    pub const fn resources(self) -> ApplicationCandidateResourceCeiling {
        self.resources
    }
}
