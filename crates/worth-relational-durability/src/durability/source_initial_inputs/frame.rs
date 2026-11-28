//! Version-first integrity framing for Native's initialization artifact.
use sha2::{Digest, Sha256};
use worth_execution::{
    decode_framed_envelope, encode_framed_bytes, ExecutionImmutableBytes, ExecutionRequest,
    FramedEnvelopeError, FramedEnvelopeSpec, FRAMED_ENVELOPE_HEADER_BYTES,
};
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity,
    BoundaryProtocolUnsupportedVersion, BoundaryProtocolVersion,
};
use worth_relational_contracts::internal_port as contracts;

pub const PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.relational.source.initial-inputs");
// Current original-input commitments bind canonical artifact v8. Older carriers
// are refused before their commitment DATA is decoded or used.
pub const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(3);
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalInitialInputsPublicationDetail {
    UnsupportedProvider,
    Request(contracts::execution::denial_cause::RelationalExecutionDenialCause),
    Physical(String),
}
const HEADER: usize = FRAMED_ENVELOPE_HEADER_BYTES + 8;
const SPEC: FramedEnvelopeSpec = FramedEnvelopeSpec {
    magic: *b"WRTHRSI1",
    supported_versions: &[VERSION.get()],
    max_payload_bytes: (isize::MAX as usize - FRAMED_ENVELOPE_HEADER_BYTES) as u64,
};

#[derive(Debug)]
pub enum RelationalInitialInputsRefusal {
    Malformed,
    Missing,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    Request(contracts::execution::denial_cause::RelationalExecutionDenialCause),
    Native(contracts::durability::data::recovery_stop::RelationalRecoveryStop),
    Source(worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal),
    Payload(contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal),
}

#[derive(Debug, Clone)]
pub struct RelationalSourceInitialInputsArtifact {
    bytes: ExecutionImmutableBytes,
    region: std::ops::Range<usize>,
    revision: u64,
    digest: [u8; 32],
}
impl RelationalSourceInitialInputsArtifact {
    pub const PROTOCOL: BoundaryProtocolIdentity = PROTOCOL;
    pub(crate) fn emit(
        revision: u64,
        payload: &impl serde::Serialize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalInitialInputsRefusal> {
        use RelationalInitialInputsRefusal as Refusal;
        let payload = contracts::durability::artifact_payload_emission::request::encode_named(
            &[],
            payload,
            request,
        )
        .map_err(Refusal::Native)?;
        let bytes = encode_framed_bytes(
            SPEC,
            VERSION.get(),
            request,
            |sink| {
                sink(&revision.to_le_bytes())?;
                sink(payload.bytes())
            },
            backing_refusal,
        )?;
        Self::from_owned(bytes, request)
    }
    pub(crate) fn from_owned(
        bytes: ExecutionImmutableBytes,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalInitialInputsRefusal> {
        let length = bytes.len();
        Self::from_region(bytes, 0..length, request)
    }
    pub(crate) fn from_region(
        bytes: ExecutionImmutableBytes,
        region: std::ops::Range<usize>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalInitialInputsRefusal> {
        use RelationalInitialInputsRefusal as Refusal;
        let raw = bytes.get(region.clone()).ok_or(Refusal::Malformed)?;
        let (_, payload) = decode_framed_envelope(SPEC, raw, request)?;
        let (revision_bytes, payload) = payload.split_at_checked(8).ok_or(Refusal::Malformed)?;
        let revision = u64::from_le_bytes(revision_bytes.try_into().unwrap());
        if revision == 0 {
            return Err(Refusal::Malformed);
        }
        // The source predecessor coordinate still identifies the original
        // input payload independently of its revision and wire envelope.
        let digest = digest(payload, request)?;
        Ok(Self {
            bytes,
            region,
            revision,
            digest,
        })
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn payload_digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(crate) fn payload(&self) -> &[u8] {
        &self.as_bytes()[HEADER..]
    }
}

fn digest(
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], RelationalInitialInputsRefusal> {
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(64 * 1024) {
        request
            .consult()
            .map_err(|cause| RelationalInitialInputsRefusal::Request(cause.into()))?;
        digest.update(chunk);
    }
    Ok(digest.finalize().into())
}

impl From<FramedEnvelopeError> for RelationalInitialInputsRefusal {
    fn from(error: FramedEnvelopeError) -> Self {
        frame_refusal(error, VERSION)
    }
}

pub(super) fn frame_refusal(
    error: FramedEnvelopeError,
    version: BoundaryProtocolVersion,
) -> RelationalInitialInputsRefusal {
    use RelationalInitialInputsRefusal as Refusal;
    match error {
        FramedEnvelopeError::Execution(cause) => Refusal::Request(cause.into()),
        FramedEnvelopeError::UnsupportedVersion { found } => {
            BoundaryProtocolVersion::try_new(found)
                .ok()
                .and_then(|found| {
                    BoundaryProtocolCompatibilityWindow::inclusive(version, version)
                        .admit(found)
                        .err()
                })
                .map_or(Refusal::Malformed, Refusal::UnsupportedVersion)
        }
        _ => Refusal::Malformed,
    }
}

pub(super) fn backing_refusal(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> RelationalInitialInputsRefusal {
    RelationalInitialInputsRefusal::Native(
        contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop(cause)
            .into(),
    )
}
