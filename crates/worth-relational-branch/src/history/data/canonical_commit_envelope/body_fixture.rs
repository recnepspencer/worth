//! Existing canonical envelope body controls for the exact courts.
use super::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody, CanonicalEnvelopeControlAllocation,
    PublishedLineageArtifact,
};

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait CanonicalCommitEnvelopeBodyFixturePort: Sized {
    fn published_lineage_mut_for_test(&mut self) -> &mut PublishedLineageArtifact;
}

impl CanonicalCommitEnvelopeBodyFixturePort for CanonicalCommitEnvelopeBody {
    fn published_lineage_mut_for_test(&mut self) -> &mut PublishedLineageArtifact {
        self.published_lineage_mut_for_test()
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait CanonicalCommitEnvelopeFixturePort: Sized {
    fn body_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelopeBody;
}

impl CanonicalCommitEnvelopeFixturePort for CanonicalCommitEnvelope {
    fn body_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelopeBody {
        self.body_mut_for_test()
    }
}

pub trait CanonicalCommitEnvelopeFieldsPort {
    fn control_allocation_for_test(
        &self,
    ) -> &Option<std::sync::Arc<CanonicalEnvelopeControlAllocation>>;
}

impl CanonicalCommitEnvelopeFieldsPort for CanonicalCommitEnvelope {
    fn control_allocation_for_test(
        &self,
    ) -> &Option<std::sync::Arc<CanonicalEnvelopeControlAllocation>> {
        &self.control_allocation
    }
}
