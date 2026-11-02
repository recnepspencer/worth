//! Version-first integrity framing for Native's initialization artifact.
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity,
    BoundaryProtocolUnsupportedVersion, BoundaryProtocolVersion,
};

pub const PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.relational.source.initial-inputs");
pub const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalInitialInputsPublicationDetail {
    UnsupportedProvider,
    Request(crate::execution::RelationalExecutionDenialCause),
    Physical(String),
}
const MAGIC: &[u8; 8] = b"WRTHRSI1";
const HEADER: usize = 60;

#[derive(Debug)]
pub enum RelationalInitialInputsRefusal {
    Malformed,
    Missing,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    Request(crate::execution::RelationalExecutionDenialCause),
    Native(crate::durability::data::RelationalRecoveryStop),
    Source(crate::durability::medium::RelationalDurableOwnedReadRefusal),
    Payload(crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal),
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
        let payload = crate::durability::artifact_payload_emission::request::encode_named(
            &[],
            payload,
            request,
        )
        .map_err(Refusal::Native)?;
        let digest = digest(payload.bytes(), request)?;
        let length = HEADER
            .checked_add(payload.len())
            .ok_or(Refusal::Malformed)?;
        let mut bytes =
            ExecutionRequestArrayBuilder::allocate(*request, length).map_err(|cause| {
                Refusal::Native(
                    crate::durability::checkpoint_history::capture::array_capture_stop(cause)
                        .into(),
                )
            })?;
        for chunk in [
            MAGIC.as_slice(),
            VERSION.get().to_le_bytes().as_slice(),
            revision.to_le_bytes().as_slice(),
            (payload.len() as u64).to_le_bytes().as_slice(),
            digest.as_slice(),
            payload.bytes(),
        ] {
            bytes.extend_from_slice(chunk).map_err(|cause| {
                Refusal::Native(
                    crate::durability::checkpoint_history::capture::array_capture_stop(cause)
                        .into(),
                )
            })?;
        }
        let bytes = bytes.seal_immutable_bytes().map_err(|cause| {
            Refusal::Native(
                crate::durability::checkpoint_history::capture::array_capture_stop(cause).into(),
            )
        })?;
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
        if raw.get(..8) != Some(MAGIC.as_slice()) {
            return Err(Refusal::Malformed);
        }
        let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
            raw.get(8..12)
                .ok_or(Refusal::Malformed)?
                .try_into()
                .map_err(|_| Refusal::Malformed)?,
        ))
        .map_err(|_| Refusal::Malformed)?;
        BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION)
            .admit(version)
            .map_err(Refusal::UnsupportedVersion)?;
        let revision = u64::from_le_bytes(
            raw.get(12..20)
                .ok_or(Refusal::Malformed)?
                .try_into()
                .map_err(|_| Refusal::Malformed)?,
        );
        let length = u64::from_le_bytes(
            raw.get(20..28)
                .ok_or(Refusal::Malformed)?
                .try_into()
                .map_err(|_| Refusal::Malformed)?,
        );
        let digest: [u8; 32] = raw
            .get(28..HEADER)
            .ok_or(Refusal::Malformed)?
            .try_into()
            .map_err(|_| Refusal::Malformed)?;
        let payload = raw.get(HEADER..).ok_or(Refusal::Malformed)?;
        if revision == 0
            || length != payload.len() as u64
            || digest != self::digest(payload, request)?
        {
            return Err(Refusal::Malformed);
        }
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
