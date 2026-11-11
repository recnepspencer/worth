//! A placement projection of the same complete sealed envelope.
use super::CanonicalCommitEnvelope;
use serde::{Serialize, Serializer};

pub(crate) struct AuthoritativeArtifactEnvelopeRef<'a> {
    envelope: &'a CanonicalCommitEnvelope,
}
impl<'a> AuthoritativeArtifactEnvelopeRef<'a> {
    pub(crate) fn new(envelope: &'a CanonicalCommitEnvelope) -> Self {
        Self { envelope }
    }
}
impl Serialize for AuthoritativeArtifactEnvelopeRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        super::wire_projection::serialize(
            self.envelope,
            serializer,
            super::wire_projection::EnvelopeProjection::Authoritative,
        )
    }
}
