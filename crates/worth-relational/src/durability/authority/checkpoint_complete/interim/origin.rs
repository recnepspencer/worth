//! Capture-issued immutable transport custody has no runtime or root keepalive.
use super::*;

#[derive(Clone, Debug)]
pub struct CapturedNativeInterimOrigin(pub(super) Arc<Origin>);
#[derive(Debug)]
pub(super) struct Origin {
    pub(super) source: Arc<CaptureIdentity>,
    pub(super) originals: ExecutionArray<crate::history::data::PositionedCanonicalCommit>,
    pub(super) image: RelationalCheckpointAdmissionArtifact,
    pub(super) wrapper: ExecutionImmutableBytes,
    pub(super) wrapper_digest: CanonicalDigestId,
    pub(super) native: Range<usize>,
    pub(super) image_region: Range<usize>,
    _memory: Option<ExecutionMemoryReservation>,
}
pub(super) fn seal(
    source: Arc<CaptureIdentity>,
    originals: ExecutionArray<crate::history::data::PositionedCanonicalCommit>,
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
    pub(in crate::durability::authority) fn native_len(&self) -> usize {
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
            crate::durability::artifact_payload_emission::request::account_work(request, 1)
                .map_err(crate::durability::checkpoint_admission::native_stop)?;
            // The sealed owner keeps this immutable allocation alive. An exact
            // slice carries that established proof; different backing hashes.
            if std::ptr::eq(bytes, self.0.wrapper.bytes()) {
                return Ok(());
            }
            if bytes.len() != self.0.wrapper.len()
                || digest(bytes, request)? != self.0.wrapper_digest
            {
                return Err(Refusal::AuthenticationMismatch(
                crate::durability::checkpoint_admission::CheckpointAdmissionAuthenticationMismatch::CaptureOrigin));
            }
            Ok(())
        })
    }
}
