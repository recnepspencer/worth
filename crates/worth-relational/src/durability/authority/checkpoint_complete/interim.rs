//! Memory capture proof is distinct from Home selection and original-frame demand.
pub(in crate::durability::authority) mod bootstrap;
mod capture;
mod component;
mod origin;
mod request_scope;
pub use bootstrap::NativeInterimBootstrapContinuation;
pub(in crate::durability::authority) mod receiver;
pub(in crate::durability::authority) mod verified;
use crate::durability::checkpoint_admission::{
    CheckpointAdmissionBindingMismatch as Mismatch, RelationalCheckpointAdmissionArtifact,
    RelationalCheckpointAdmissionRefusal as Refusal,
};
use crate::history::{RelationalCommitArtifact, RetainedCanonicalIndex};
pub use component::CapturedNativeInterimComponent;
pub use origin::CapturedNativeInterimOrigin;
pub use receiver::AdmittedNativeInterimReceiver;
use std::{
    ops::Range,
    sync::{Arc, Weak},
};
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionOrderedMap,
    ExecutionRequest,
};
use worth_foundational::{
    facade::{CheckpointAdmissionCut, CheckpointAdmissionScope},
    CanonicalDigestId,
};

#[derive(Debug)]
pub(super) struct CaptureIdentity {
    coordinate: CanonicalDigestId,
    canonical: ExecutionOrderedMap<crate::history::data::CommitId, RetainedCanonicalIndex>,
    _memory: Option<ExecutionMemoryReservation>,
}

/// Only the actual Memory verifier can issue this staging owner. It has no
/// byte/field constructor and is consumed when its final transport is sealed.
pub struct CapturedNativeInterimAdmission {
    source: Arc<CaptureIdentity>,
    originals: ExecutionArray<crate::history::data::PositionedCanonicalCommit>,
    rows: ExecutionArray<crate::durability::checkpoint_admission::EncodedAdmissionRow>,
    roots: ExecutionImmutableBytes,
    _artifacts: ExecutionArray<Arc<RelationalCommitArtifact>>,
    current: crate::branch::CheckpointBranchMetadata,
    owner: Weak<crate::runtime::RelationalRuntimeState>,
    native_digest: CanonicalDigestId,
    covered: u64,
    transitions: u64,
    emitted: Option<RelationalCheckpointAdmissionArtifact>,
}
impl std::fmt::Debug for CapturedNativeInterimAdmission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedNativeInterimAdmission")
            .field("covered", &self.covered)
            .finish_non_exhaustive()
    }
}
impl CapturedNativeInterimAdmission {
    fn prepare(
        verified: worth_proof::NativeCheckpointOriginalsVerified<
            verified::VerifiedMemoryProjection,
        >,
        coordinate: CanonicalDigestId,
        digest: [u8; 32],
        covered: u64,
        transitions: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::Denial> {
        let memory =
            reserve_arc::<CaptureIdentity>(request).map_err(super::Denial::AdmissionImage)?;
        verified.consume(|verified| {
            Ok(Self {
                source: Arc::new(CaptureIdentity {
                    coordinate,
                    canonical: verified.canonical,
                    _memory: memory,
                }),
                originals: verified.originals,
                rows: verified.rows,
                roots: verified.roots,
                _artifacts: verified.artifacts,
                current: verified.current,
                owner: verified.owner,
                native_digest: CanonicalDigestId::new(digest),
                covered,
                transitions,
                emitted: None,
            })
        })
    }
    pub fn coordinate(&self) -> CanonicalDigestId {
        self.source.coordinate
    }
    pub fn native_digest(&self) -> CanonicalDigestId {
        self.native_digest
    }
    pub fn covered_position(&self) -> u64 {
        self.covered
    }
    pub fn covered_transitions(&self) -> u64 {
        self.transitions
    }

    /// World supplies DATA affinity, never authority. Its own capture origin
    /// separately authenticates actual committed World movement and receiver.
    pub fn emit_interim_image(
        &mut self,
        registry_revision: u64,
        world_snapshot_body_digest: CanonicalDigestId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalCheckpointAdmissionArtifact, Refusal> {
        request_scope::run(request, || {
            request
                .consult()
                .map_err(|cause| Refusal::Request(cause.into()))?;
            if self.emitted.is_some() {
                return Err(Refusal::OwnerUnavailable);
            }
            let cut = CheckpointAdmissionCut::new(
                CheckpointAdmissionScope::Interim {
                    transport_origin: *self.source.coordinate.bytes(),
                    registry_revision,
                    world_snapshot_body_digest: *world_snapshot_body_digest.bytes(),
                },
                self.covered,
                self.transitions,
                *self.native_digest.bytes(),
            );
            let image = crate::durability::checkpoint_admission::emit(
                &cut,
                &self.roots,
                &self.rows,
                request,
            )?;
            self.emitted = Some(image.clone());
            Ok(image)
        })
    }

    /// Seal the whole immutable wrapper and both exact Native children. A
    /// caller-supplied digest or a parsed image cannot issue this origin.
    pub fn seal_transport(
        self,
        wrapper: ExecutionImmutableBytes,
        native: Range<usize>,
        image: Range<usize>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimOrigin, Refusal> {
        request_scope::run(request, || {
            let emitted = self.emitted.ok_or(Refusal::MissingArtifact)?;
            let native_bytes = wrapper
                .bytes()
                .get(native.clone())
                .ok_or(Refusal::Malformed)?;
            let image_bytes = wrapper
                .bytes()
                .get(image.clone())
                .ok_or(Refusal::Malformed)?;
            crate::durability::artifact_payload_emission::request::account_work(
                request,
                image_bytes.len() as u64,
            )
            .map_err(crate::durability::checkpoint_admission::native_stop)?;
            if digest(native_bytes, request)? != self.native_digest
                || image_bytes != emitted.as_bytes()
            {
                return Err(Refusal::AuthenticationMismatch(
                crate::durability::checkpoint_admission::CheckpointAdmissionAuthenticationMismatch::CaptureOrigin));
            }
            origin::seal(
                self.source,
                self.originals,
                emitted,
                wrapper,
                native,
                image,
                request,
            )
        })
    }
}

impl crate::durability::authority::DurabilityAuthority<'_> {
    pub fn capture_interim_checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            crate::durability::data::RelationalNativeCheckpoint,
            CapturedNativeInterimAdmission,
        ),
        super::Denial,
    > {
        policy.check_live()?;
        crate::execution::check_request(request).map_err(super::Denial::Request)?;
        capture::capture(self.runtime, request)
    }
}

pub(super) fn reserve_arc<T>(
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, Refusal> {
    let layout = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| Refusal::CapacityOverflow)?
        .0
        .pad_to_align();
    request
        .reserve_retained_memory(layout.size() as u64)
        .map_err(|cause| Refusal::Request(cause.into()))
}
pub(super) fn digest(
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<CanonicalDigestId, Refusal> {
    crate::durability::checkpoint_admission::binding_digest(bytes, request)
        .map(CanonicalDigestId::new)
}
