//! One originally funded canonical frame admitted by its digest and fixed header.
use super::body_read::complete;
use contracts::durability::canonical_commit_artifact::frame::CanonicalCommitArtifactHeader;
use contracts::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial;
use contracts::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal;
use contracts::durability::canonical_commit_artifact::payload_digest::{
    RelationalCanonicalCommitArtifactDigest as Digest,
    RelationalCanonicalCommitArtifactDigestOwnerPort as _,
};
use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal;
use contracts::execution::request_stage::{checkpoint, optional_stage};
use contracts::history::data::coordinates::CommitId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_execution::{
    ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial,
};
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_contracts::internal_port as contracts;

#[derive(Clone, Debug)]
pub struct VerifiedCanonicalCommitFrame {
    pub(super) bytes: ExecutionImmutableBytes,
    pub(super) header: CanonicalCommitArtifactHeader,
}

impl VerifiedCanonicalCommitFrame {
    pub(crate) fn from_owned_bytes(
        bytes: ExecutionImmutableBytes,
    ) -> Result<Self, RelationalCanonicalCommitArtifactFrameRefusal> {
        let (header, _) = CanonicalCommitArtifactHeader::verify(bytes.bytes())?;
        Ok(Self { bytes, header })
    }

    pub fn position(&self) -> PatchStreamPosition {
        self.header.position
    }
    pub(crate) fn commit_id(&self) -> CommitId {
        self.header.commit_id
    }
    pub fn payload_digest(&self) -> Digest {
        self.header.digest
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
    pub(crate) fn into_bytes(self) -> ExecutionImmutableBytes {
        self.bytes
    }

    pub fn reference_encode_for_test(
        commit: &PositionedCanonicalCommit,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, contracts::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial>{
        super::frame_reference_encoding::encode(commit, policy)
    }

    pub(crate) fn from_owned_original(
        bytes: worth_execution::ExecutionImmutableBytes,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, ControlledFrameCopyDenial> {
        let mut answer = None;
        let handoff = optional_stage(
            execution,
            |context, _| {
                answer = Some((|| {
                    let (header, payload) = CanonicalCommitArtifactHeader::inspect(bytes.bytes())
                        .map_err(ControlledFrameCopyDenial::Frame)?;
                    let digest = Digest::of_payload_for_request(
                        header.position.0,
                        header.commit_id.0,
                        payload,
                        context,
                    )
                    .map_err(copy_digest)?;
                    header
                        .require_digest(digest)
                        .map_err(ControlledFrameCopyDenial::Frame)?;
                    Ok(header)
                })());
            },
            |_| 0,
        );
        let header = match (handoff, answer) {
            (Ok(()), Some(answer)) => answer?,
            (Err(_), Some(Err(cause))) => return Err(cause),
            (Err(cause), _) => return Err(ControlledFrameCopyDenial::Request(cause)),
            (Ok(()), None) => unreachable!("completed original verification ran its callback"),
        };
        Ok(Self { bytes, header })
    }

    pub(crate) fn copy_frame_original(
        input: &[u8],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, ControlledFrameCopyDenial> {
        let mut answer = None;
        let handoff = optional_stage(
            execution,
            |context, _| {
                answer = Some((|| {
                    let (header, payload) = CanonicalCommitArtifactHeader::inspect(input)
                        .map_err(ControlledFrameCopyDenial::Frame)?;
                    let digest = Digest::of_payload_for_request(
                        header.position.0,
                        header.commit_id.0,
                        payload,
                        context,
                    )
                    .map_err(copy_digest)?;
                    header
                        .require_digest(digest)
                        .map_err(ControlledFrameCopyDenial::Frame)?;
                    let mut bytes =
                        ExecutionRequestArrayBuilder::<u8>::allocate(*execution, input.len())
                            .map_err(copy_allocation)?;
                    for chunk in input.chunks(4096) {
                        checkpoint(context, chunk.len() as u64)
                            .map_err(|cause| ControlledFrameCopyDenial::Request(cause.into()))?;
                        bytes.extend_from_slice(chunk).map_err(copy_allocation)?;
                    }
                    Ok(Self {
                        header,
                        bytes: bytes
                            .seal()
                            .map_err(copy_allocation)?
                            .into_immutable_bytes(),
                    })
                })());
            },
            |_| 0,
        );
        match handoff {
            Ok(()) => answer.expect("completed original frame copy ran its callback"),
            Err(cause) => match answer {
                Some(Err(cause)) => Err(cause),
                _ => Err(ControlledFrameCopyDenial::Request(cause)),
            },
        }
    }
    /// The actual immutable frame already proved its digest. Preserve its
    /// original byte allocation while admitting every decoded owner field.
    pub fn readmitted_body(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, Refusal> {
        contracts::execution::request_custody::check_request(execution)
            .map_err(Refusal::Request)?;
        let mut answer = None;
        let handoff = contracts::execution::request_stage::optional_stage(
            execution,
            |context, _| {
                answer = Some(super::body_read::readmit_canonical_body(
                    &self.header,
                    &self.as_bytes()
                        [contracts::durability::canonical_commit_artifact::frame::HEADER_LEN..],
                    execution,
                    context,
                    Some(self.bytes.clone()),
                ));
            },
            |_| 0,
        );
        complete(handoff, answer)
    }
}

fn copy_allocation(cause: ExecutionRequestArrayDenial) -> ControlledFrameCopyDenial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => {
            ControlledFrameCopyDenial::Request(cause.into())
        }
        ExecutionRequestArrayDenial::Allocation(cause) => {
            ControlledFrameCopyDenial::Allocation(cause)
        }
    }
}

fn copy_digest(
    cause: contracts::durability::canonical_commit_artifact::payload_digest::RequestDigestDenial,
) -> ControlledFrameCopyDenial {
    match cause {
        contracts::durability::canonical_commit_artifact::payload_digest::RequestDigestDenial::Frame(cause) => {
            ControlledFrameCopyDenial::Frame(cause)
        }
        contracts::durability::canonical_commit_artifact::payload_digest::RequestDigestDenial::Request(cause) => {
            ControlledFrameCopyDenial::Request(cause)
        }
    }
}

pub mod indexed;
