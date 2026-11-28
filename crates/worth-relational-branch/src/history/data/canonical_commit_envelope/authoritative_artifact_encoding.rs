//! A placement projection of the same complete sealed envelope.
use super::wire_projection::EnvelopeProjection;
use super::CanonicalCommitEnvelope;
use serde::{Serialize, Serializer};
use worth_relational_contracts::internal_port::durability::canonical_commit_artifact::frame::CanonicalCommitArtifactFormat;

pub struct AuthoritativeArtifactEnvelopeRef<'a> {
    envelope: &'a CanonicalCommitEnvelope,
    projection: EnvelopeProjection,
}
impl<'a> AuthoritativeArtifactEnvelopeRef<'a> {
    pub fn new(envelope: &'a CanonicalCommitEnvelope) -> Self {
        Self::for_format(envelope, envelope.artifact_format())
    }

    pub(crate) fn for_format(
        envelope: &'a CanonicalCommitEnvelope,
        format: CanonicalCommitArtifactFormat,
    ) -> Self {
        let projection = match format {
            CanonicalCommitArtifactFormat::Ordinary => EnvelopeProjection::Authoritative,
            CanonicalCommitArtifactFormat::LegacyMergeCapable => {
                EnvelopeProjection::LegacyAuthoritative
            }
        };
        Self {
            envelope,
            projection,
        }
    }
}
impl Serialize for AuthoritativeArtifactEnvelopeRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        super::wire_projection::serialize(self.envelope, serializer, self.projection)
    }
}
