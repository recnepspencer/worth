use worth_execution::ExecutionRequest;

use super::frame::CanonicalCommitArtifactHeader;
use super::payload_wire::{ArtifactPayloadRef, ArtifactPayloadWire};
use super::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use crate::durability::artifact_canonical_payload::is_canonical_payload;
use crate::durability::migration::ReadmittedCanonicalCommit;

impl RelationalCanonicalCommitArtifact {
    /// The only door from artifact bytes back to canonical authority.
    ///
    /// Phases, cheapest first, each refusing before the next runs: request
    /// liveness; frame, version, length and digest; payload decode; header
    /// agreement with the payload position and envelope commit id; managed
    /// field-text readmission; positioned-commit admission; and finally a
    /// canonical-form check that the payload is exactly the admitted
    /// commit's encoding.
    ///
    /// Cost: O(payload bytes) for the digest, decode and canonical check,
    /// plus field-text admission charged to `execution`.
    pub fn readmit(
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        let (header, payload) = CanonicalCommitArtifactHeader::verify(bytes)?;
        let wire = decode_payload(payload)?;
        check_header_agreement(&header, &wire)?;
        let envelope = wire
            .canonical
            .readmit(execution)
            .map_err(Refusal::FieldText)?;
        let readmitted = ReadmittedCanonicalCommit::readmit_current(header.position, envelope)
            .map_err(Refusal::PositionAdmission)?;
        let canonical = ArtifactPayloadRef::new(readmitted.position(), readmitted.envelope());
        if !is_canonical_payload(&canonical, payload) {
            return Err(Refusal::NonCanonicalPayload);
        }
        Ok(readmitted.with_verified_journal_payload(*header.digest.as_bytes()))
    }
}

fn decode_payload(payload: &[u8]) -> Result<ArtifactPayloadWire<'_>, Refusal> {
    rmp_serde::from_slice(payload).map_err(|error| Refusal::PayloadDecode {
        detail: error.to_string(),
    })
}

fn check_header_agreement(
    header: &CanonicalCommitArtifactHeader,
    wire: &ArtifactPayloadWire<'_>,
) -> Result<(), Refusal> {
    if header.position != wire.position {
        return Err(Refusal::PositionDisagreement {
            header: header.position,
            payload: wire.position,
        });
    }
    let envelope_commit = wire.canonical.commit_id();
    if header.commit_id != envelope_commit {
        return Err(Refusal::CommitIdDisagreement {
            header: header.commit_id,
            envelope: envelope_commit,
        });
    }
    Ok(())
}
