//! Capture-issued immutable transport custody has no runtime or root keepalive.
use super::*;
use worth_relational_contracts::internal_port as contracts;

#[derive(Clone, Debug)]
pub struct CapturedNativeInterimOrigin(pub(super) Arc<Origin>);
#[derive(Debug)]
pub struct Origin {
    pub source: Arc<CaptureIdentity>,
    pub originals: ExecutionArray<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
    pub image: RelationalCheckpointAdmissionArtifact,
    pub wrapper: ExecutionImmutableBytes,
    pub wrapper_digest: CanonicalDigestId,
    pub native: Range<usize>,
    pub image_region: Range<usize>,
    _memory: Option<ExecutionMemoryReservation>,
}
pub(super) fn seal(
    source: Arc<CaptureIdentity>,
    originals: ExecutionArray<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
    image: RelationalCheckpointAdmissionArtifact,
    wrapper: ExecutionImmutableBytes,
    native: Range<usize>,
    image_region: Range<usize>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CapturedNativeInterimOrigin, Refusal> {
    let memory = reserve_arc::<Origin>(request)?;
    let wrapper_digest = digest(wrapper.bytes(), request)?;
    Ok(CapturedNativeInterimOrigin(Arc::new(Origin {
        source,
        originals,
        image,
        wrapper,
        wrapper_digest,
        native,
        image_region,
        _memory: memory,
    })))
}
impl CapturedNativeInterimOrigin {
    pub(in crate::internal_port::durability::authority) fn native_len(&self) -> usize {
        self.0.native.len()
    }
    pub fn coordinate(&self) -> CanonicalDigestId {
        self.0.source.coordinate
    }
    pub fn admission_cut(&self) -> &CheckpointAdmissionCut {
        self.0.image.cut()
    }
    pub fn wrapper_digest(&self) -> CanonicalDigestId {
        self.0.wrapper_digest
    }
    pub fn validate_transport(
        &self,
        bytes: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        request_scope::run(request, || {
            contracts::durability::data::recovery_stop::account_work(request, 1)
                .map_err(contracts::durability::checkpoint_admission::refusal::native_stop)?;
            // The sealed owner keeps this immutable allocation alive. An exact
            // slice carries that established proof; different backing hashes.
            if std::ptr::eq(bytes, self.0.wrapper.bytes()) {
                return Ok(());
            }
            if bytes.len() != self.0.wrapper.len()
                || digest(bytes, request)? != self.0.wrapper_digest
            {
                return Err(Refusal::AuthenticationMismatch(
                contracts::durability::checkpoint_admission::refusal::CheckpointAdmissionAuthenticationMismatch::CaptureOrigin));
            }
            Ok(())
        })
    }
}

#[doc(hidden)]
pub trait CapturedNativeInterimOriginOwnerPort: Sized {
    fn original_capture(&self) -> &Origin;

    fn native_len(&self) -> usize;
}

impl CapturedNativeInterimOriginOwnerPort for CapturedNativeInterimOrigin {
    fn original_capture(&self) -> &Origin {
        &self.0
    }

    fn native_len(&self) -> usize {
        self.native_len()
    }
}
