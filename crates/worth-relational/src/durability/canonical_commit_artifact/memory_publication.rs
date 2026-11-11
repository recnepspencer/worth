//! Issue one original frame from genuine producer data, before performed admission.
use super::frame::{CanonicalCommitArtifactHeader, HEADER_LEN};
use super::payload_wire::ArtifactPayloadRef;
use super::PreparedOriginalCanonicalPayload;
use super::{
    RelationalCanonicalCommitArtifact as Artifact,
    RelationalCanonicalCommitArtifactEncodeDenial as Denial,
};
use crate::publication::patch::data::PatchStreamPosition;
use std::io::{self, Write};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
mod denial;
#[cfg(test)]
mod size_tests;
pub(crate) use denial::MemoryFramePreparationDenial;

impl Artifact {
    /// The position is reserved producer DATA. This issues bytes, not performed proof.
    pub(crate) fn from_memory_publication(
        position: PatchStreamPosition,
        prepared: &PreparedOriginalCanonicalPayload,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, MemoryFramePreparationDenial> {
        let envelope = prepared.envelope();
        let payload = ArtifactPayloadRef::new(position, envelope);
        let mut count = Sizing {
            request: *request,
            length: 0,
            failure: None,
        };
        let result = rmp_serde::encode::write_named(&mut count, &payload);
        settle(result, count.failure.take())?;
        let payload_len = count.length;
        let frame_len = bounded_frame_len(payload_len)?;
        let header = CanonicalCommitArtifactHeader {
            position,
            commit_id: envelope.commit.commit_id,
            digest: prepared.at_position(position),
        };
        let mut buffer = ExecutionRequestArrayBuilder::<u8>::allocate(*request, frame_len)
            .map_err(MemoryFramePreparationDenial::from)?;
        buffer
            .extend_from_slice(
                &header.emit(u64::try_from(payload_len).map_err(|_| Denial::FrameSizeOverflow)?),
            )
            .map_err(MemoryFramePreparationDenial::from)?;
        let mut emission = Emission {
            buffer: &mut buffer,
            failure: None,
        };
        let result = rmp_serde::encode::write_named(&mut emission, &payload);
        settle(result, emission.failure.take())?;
        if buffer.len() != frame_len {
            return Err(Denial::CountEmissionMismatch {
                counted: payload_len,
                emitted: buffer.len().saturating_sub(HEADER_LEN),
            }
            .into());
        }
        let bytes = buffer
            .seal_immutable_bytes()
            .map_err(MemoryFramePreparationDenial::from)?;
        // These bytes were deterministically emitted from the exact immutable
        // owner whose two leaves were prepared, with count/emission agreement.
        // External bytes still pass the full frame verification door.
        let artifact = Self { bytes, header };
        request
            .consult()
            .map_err(|cause| MemoryFramePreparationDenial::Request(cause))?;
        Ok(artifact)
    }
}
struct Sizing<'r, 'a> {
    request: ExecutionRequest<'r, 'a>,
    length: usize,
    failure: Option<MemoryFramePreparationDenial>,
}
impl Write for Sizing<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = match self
            .length
            .checked_add(bytes.len())
            .filter(|len| *len <= isize::MAX as usize)
        {
            Some(next) => next,
            None => {
                self.failure = Some(Denial::FrameSizeOverflow.into());
                return Err(stopped());
            }
        };
        if let Err(cause) = bounded_frame_len(next) {
            self.failure = Some(cause.into());
            return Err(stopped());
        }
        for _ in bytes.chunks(64 * 1024) {
            if let Err(cause) = self.request.consult() {
                self.failure = Some(MemoryFramePreparationDenial::Request(cause));
                return Err(stopped());
            }
        }
        self.length = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Emission<'b, 'r, 'a> {
    buffer: &'b mut ExecutionRequestArrayBuilder<'r, 'a, u8>,
    failure: Option<MemoryFramePreparationDenial>,
}
impl Write for Emission<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.buffer.extend_from_slice(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(cause) => {
                self.failure = Some(cause.into());
                Err(stopped())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn settle(
    result: Result<(), rmp_serde::encode::Error>,
    failure: Option<MemoryFramePreparationDenial>,
) -> Result<(), MemoryFramePreparationDenial> {
    if let Some(failure) = failure {
        return Err(failure);
    }
    result.map_err(|cause| {
        MemoryFramePreparationDenial::Artifact(Denial::PayloadEncoding {
            detail: cause.to_string(),
        })
    })
}
fn stopped() -> io::Error {
    io::Error::other("original Memory artifact emission stopped")
}

fn bounded_frame_len(payload: usize) -> Result<usize, Denial> {
    let observed = HEADER_LEN
        .checked_add(payload)
        .ok_or(Denial::FrameSizeOverflow)?;
    if observed > super::MAXIMUM_ARTIFACT_BYTES {
        return Err(Denial::SizeLimit {
            observed,
            maximum: super::MAXIMUM_ARTIFACT_BYTES,
        });
    }
    Ok(observed)
}
