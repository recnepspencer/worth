//! Durable cutover retains the genuine request through count, emit and hash.
use super::frame_contract::MAXIMUM_ARTIFACT_BYTES;
use super::{
    encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial,
    frame::{CanonicalCommitArtifactHeader, DIGEST_OFFSET, HEADER_LEN},
    payload_digest::RelationalCanonicalCommitArtifactDigest as Digest,
    payload_wire::ArtifactPayloadRef,
    verified_frame::VerifiedCanonicalCommitFrame,
};
use crate::{
    execution::request_stage::{checkpoint, optional_stage},
    history::data::PositionedCanonicalCommit,
};
use std::io::{self, Write};
use worth_execution::{
    ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial, MapKernelContext,
};

pub(crate) fn encode_canonical_frame_for_request(
    commit: &PositionedCanonicalCommit,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<VerifiedCanonicalCommitFrame, Denial> {
    let mut answer = None;
    let handoff = optional_stage(
        execution,
        |context, _| {
            answer = Some(encode(commit, execution, context));
        },
        |_| 0,
    );
    match handoff {
        Ok(()) => answer.expect("completed artifact stage ran its callback"),
        Err(cause) => match answer {
            Some(Err(cause)) => Err(cause),
            _ => Err(Denial::Request(cause)),
        },
    }
}

fn encode(
    commit: &PositionedCanonicalCommit,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<VerifiedCanonicalCommitFrame, Denial> {
    let payload = ArtifactPayloadRef::new(commit.position(), commit.envelope());
    let length = emit(&payload, context, None)?;
    let total = HEADER_LEN
        .checked_add(length)
        .ok_or(Denial::FrameSizeOverflow)?;
    if total > MAXIMUM_ARTIFACT_BYTES {
        return Err(Denial::SizeLimit {
            observed: total,
            maximum: MAXIMUM_ARTIFACT_BYTES,
        });
    }
    let header = CanonicalCommitArtifactHeader {
        position: commit.position(),
        commit_id: commit.commit.commit_id,
        digest: Digest::unsealed(),
    };
    let mut bytes =
        ExecutionRequestArrayBuilder::<u8>::allocate(*execution, total).map_err(allocation)?;
    checkpoint(context, HEADER_LEN as u64).map_err(|cause| Denial::Request(cause.into()))?;
    bytes
        .extend_from_slice(&header.emit(length as u64))
        .map_err(allocation)?;
    let emitted = emit(&payload, context, Some(&mut bytes))?;
    if emitted != length || bytes.len() != total {
        return Err(Denial::CountEmissionMismatch {
            counted: length,
            emitted,
        });
    }
    let digest = Digest::of_payload_for_request(
        header.position.0,
        header.commit_id.0,
        &bytes.elements()[HEADER_LEN..],
        context,
    )
    .map_err(encode_digest)?;
    for (index, byte) in digest.as_bytes().iter().enumerate() {
        bytes
            .replace_initialized(DIGEST_OFFSET + index, *byte)
            .map_err(allocation)?;
    }
    checkpoint(context, 0).map_err(|cause| Denial::Request(cause.into()))?;
    Ok(VerifiedCanonicalCommitFrame {
        bytes: bytes.seal().map_err(allocation)?.into_immutable_bytes(),
        header: CanonicalCommitArtifactHeader { digest, ..header },
    })
}

fn emit(
    payload: &ArtifactPayloadRef<'_>,
    context: &mut MapKernelContext<'_, '_>,
    output: Option<&mut ExecutionRequestArrayBuilder<'_, '_, u8>>,
) -> Result<usize, Denial> {
    let mut writer = OriginalPayloadWriter {
        context,
        output,
        count: 0,
        failure: None,
    };
    let result = rmp_serde::encode::write_named(&mut writer, payload);
    if let Some(cause) = writer.failure {
        return Err(cause);
    }
    result.map_err(|error| Denial::PayloadEncoding {
        detail: error.to_string(),
    })?;
    Ok(writer.count)
}

struct OriginalPayloadWriter<'a, 'scope, 'lease, 'output, 'request, 'authority> {
    context: &'a mut MapKernelContext<'scope, 'lease>,
    output: Option<&'output mut ExecutionRequestArrayBuilder<'request, 'authority, u8>>,
    count: usize,
    failure: Option<Denial>,
}
impl OriginalPayloadWriter<'_, '_, '_, '_, '_, '_> {
    fn append(&mut self, input: &[u8]) -> Result<(), Denial> {
        self.count = self
            .count
            .checked_add(input.len())
            .ok_or(Denial::FrameSizeOverflow)?;
        let total = self
            .count
            .checked_add(HEADER_LEN)
            .ok_or(Denial::FrameSizeOverflow)?;
        if total > MAXIMUM_ARTIFACT_BYTES {
            return Err(Denial::SizeLimit {
                observed: total,
                maximum: MAXIMUM_ARTIFACT_BYTES,
            });
        }
        for chunk in input.chunks(4096) {
            checkpoint(self.context, chunk.len() as u64)
                .map_err(|cause| Denial::Request(cause.into()))?;
            if let Some(output) = &mut self.output {
                output.extend_from_slice(chunk).map_err(allocation)?;
            }
        }
        Ok(())
    }
}
impl Write for OriginalPayloadWriter<'_, '_, '_, '_, '_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other("original artifact request stopped"));
        }
        match self.append(input) {
            Ok(()) => Ok(input.len()),
            Err(cause) => {
                self.failure = Some(cause);
                Err(io::Error::other("original artifact request stopped"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn allocation(cause: ExecutionRequestArrayDenial) -> Denial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => Denial::Request(cause.into()),
        ExecutionRequestArrayDenial::Allocation(cause) => Denial::Allocation(cause),
    }
}

fn encode_digest(cause: super::payload_digest::RequestDigestDenial) -> Denial {
    match cause {
        super::payload_digest::RequestDigestDenial::Frame(cause) => Denial::PayloadEncoding {
            detail: cause.to_string(),
        },
        super::payload_digest::RequestDigestDenial::Request(cause) => Denial::Request(cause),
    }
}
