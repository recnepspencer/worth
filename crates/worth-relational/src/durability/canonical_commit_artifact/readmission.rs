use worth_execution::ExecutionRequest;

use super::frame::CanonicalCommitArtifactHeader;
use super::payload_wire::{ArtifactPayloadRef, ArtifactPayloadWire};
use super::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use crate::durability::artifact_canonical_payload::is_canonical_payload_with_original_context;
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
        let mut answer = None;
        let handoff = crate::execution::request_stage::optional_stage(
            execution,
            |context, _| {
                answer = Some(Self::readmit_in_context(bytes, execution, context));
            },
            |_| 0,
        );
        match handoff {
            Ok(()) => answer.expect("completed artifact readmission ran its callback"),
            Err(cause) => match answer {
                Some(Err(cause)) => Err(cause),
                _ => Err(Refusal::Request(cause)),
            },
        }
    }

    fn readmit_in_context(
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        let (header, payload) = CanonicalCommitArtifactHeader::inspect(bytes)?;
        let digest =
            super::payload_digest::RelationalCanonicalCommitArtifactDigest::of_payload_for_request(
                payload, context,
            )
            .map_err(Refusal::Request)?;
        header.require_digest(digest)?;
        Self::readmit_payload(&header, payload, execution, context)
    }

    /// The skip is selected by this actual sealed immutable frame, never a
    /// recorded digest/header supplied separately from its backing.
    pub(crate) fn readmit_framed(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        let mut answer = None;
        let handoff = crate::execution::request_stage::optional_stage(
            execution,
            |context, _| {
                answer = Some(Self::readmit_payload(
                    &self.header,
                    &self.as_bytes()[super::frame::HEADER_LEN..],
                    execution,
                    context,
                ));
            },
            |_| 0,
        );
        match handoff {
            Ok(()) => answer.expect("completed framed readmission ran its callback"),
            Err(cause) => match answer {
                Some(Err(cause)) => Err(cause),
                _ => Err(Refusal::Request(cause)),
            },
        }
    }

    fn readmit_payload(
        header: &CanonicalCommitArtifactHeader,
        payload: &[u8],
        execution: &ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        let wire = decode_payload(payload, execution)?;
        check_header_agreement(header, &wire)?;
        let envelope = wire
            .canonical
            .readmit(execution)
            .map_err(Refusal::FieldText)?;
        let readmitted = ReadmittedCanonicalCommit::readmit_current(header.position, envelope)
            .map_err(Refusal::PositionAdmission)?;
        let canonical = ArtifactPayloadRef::new(readmitted.position(), readmitted.envelope());
        if !is_canonical_payload_with_original_context(&canonical, payload, context)
            .map_err(Refusal::Request)?
        {
            return Err(Refusal::NonCanonicalPayload);
        }
        Ok(readmitted.with_verified_journal_payload(*header.digest.as_bytes()))
    }
}

fn decode_payload<'de>(
    payload: &'de [u8],
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ArtifactPayloadWire<'de>, Refusal> {
    use serde::de::DeserializeSeed;
    let context = crate::content::decoding::ContentDecodeContext::new(execution);
    let mut decoder = rmp_serde::Deserializer::from_read_ref(payload);
    let result = crate::history::data::PositionedEnvelopeSeed(&context).deserialize(&mut decoder);
    if let Some(cause) = context.take_stop() {
        return Err(Refusal::Content(cause));
    }
    let (position, canonical) = result.map_err(|error| Refusal::PayloadDecode {
        detail: error.to_string(),
    })?;
    Ok(ArtifactPayloadWire {
        position,
        canonical,
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
