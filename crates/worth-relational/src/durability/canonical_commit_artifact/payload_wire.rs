//! The digested artifact payload: the stream position bound to the
//! authoritative envelope projection. Emission borrows the live envelope;
//! decoding borrows the payload bytes and grants nothing until readmission.

use serde::{Deserialize, Serialize};

use crate::history::data::{
    AuthoritativeArtifactEnvelopeRef, CanonicalCommitEnvelope, CanonicalEnvelopeWire,
};
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Serialize)]
pub(super) struct ArtifactPayloadRef<'a> {
    position: PatchStreamPosition,
    canonical: AuthoritativeArtifactEnvelopeRef<'a>,
}

impl<'a> ArtifactPayloadRef<'a> {
    pub(super) fn new(
        position: PatchStreamPosition,
        envelope: &'a CanonicalCommitEnvelope,
    ) -> Self {
        Self {
            position,
            canonical: AuthoritativeArtifactEnvelopeRef::new(envelope),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactPayloadWire<'a> {
    pub(super) position: PatchStreamPosition,
    #[serde(borrow)]
    pub(super) canonical: CanonicalEnvelopeWire<'a>,
}
