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
    original_inputs:
        Option<&'a crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs>,
}

impl<'a> ArtifactPayloadRef<'a> {
    pub(super) fn new(
        position: PatchStreamPosition,
        envelope: &'a CanonicalCommitEnvelope,
    ) -> Self {
        Self {
            position,
            canonical: AuthoritativeArtifactEnvelopeRef::new(envelope),
            original_inputs: envelope.recorded_original_inputs(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactPayloadWire<'a> {
    pub(super) position: PatchStreamPosition,
    #[serde(borrow)]
    pub(super) canonical: CanonicalEnvelopeWire<'a>,
    #[serde(deserialize_with = "required_original_inputs")]
    pub(super) original_inputs:
        Option<crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs>,
}

fn required_original_inputs<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<
    Option<crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs>,
    D::Error,
> {
    Option::deserialize(deserializer)
}
