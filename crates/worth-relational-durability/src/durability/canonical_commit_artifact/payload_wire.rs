//! The digested artifact payload: the stream position bound to the
//! authoritative envelope projection. Emission borrows the live envelope;
//! decoding borrows the payload bytes and grants nothing until readmission.

use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use serde::Serialize;
use worth_relational_branch::internal_port as branch;
pub mod seed;
pub(super) use seed::decode_payload;

use branch::history::data::canonical_commit_envelope::authoritative_artifact_encoding::AuthoritativeArtifactEnvelopeRef;
use branch::history::data::canonical_commit_envelope::text_wire::CanonicalEnvelopeWire;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;

#[derive(Serialize)]
pub(super) struct ArtifactPayloadRef<'a> {
    position: PatchStreamPosition,
    canonical: AuthoritativeArtifactEnvelopeRef<'a>,
    original_inputs:
        Option<&'a branch::history::data::canonical_commit_envelope::recorded_inputs::RecordedCanonicalInputs>,
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
    pub(super) canonical: CanonicalEnvelopeWire<
        worth_relational_contracts::internal_port::symbols::data::text_wire::SymbolSnapshotWire<'a>,
    >,
    pub(super) original_inputs: Option<
        branch::history::data::canonical_commit_envelope::recorded_inputs::RecordedCanonicalInputs,
    >,
}
