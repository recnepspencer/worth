//! The digested artifact payload: the stream position bound to the
//! authoritative envelope projection. Emission borrows the live envelope;
//! decoding borrows the payload bytes and grants nothing until readmission.

use serde::Serialize;
mod seed;
pub(super) use seed::decode_payload;

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

pub(super) struct ArtifactPayloadWire<'a> {
    pub(super) position: PatchStreamPosition,
    pub(super) canonical: CanonicalEnvelopeWire<crate::symbols::data::SymbolSnapshotWire<'a>>,
    pub(super) original_inputs:
        Option<crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs>,
}
