//! Memory-only checkpoint originals, captured and admitted by the actual World.
mod capture;
mod continuation;
mod request_scope;
mod restore;
mod wire;
use crate::branch::ProductBranchObservation;
use crate::history::{MemoryPublicationMeaning, MemoryPublicationReceiverScope};
use std::{
    ops::Range,
    sync::{Arc, Weak},
};
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest,
};
use worth_foundational::{facade::MemoryProductBranchReference, CanonicalDigestId};
use worth_relational::facade::durability::{
    CapturedNativeInterimComponent, CapturedNativeInterimOrigin,
};

#[derive(Debug)]
pub enum WorldInterimCheckpointRefusal {
    OwnerUnavailable,
    CaptureChanged,
    BindingMismatch,
    AuthenticationMismatch,
    MissingCaptureOrigin,
    ReceiverAlreadyBound,
    AllocationFailed,
    SizeOverflow,
    Malformed,
    Request(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Native(worth_relational::facade::durability::RelationalCheckpointAdmissionRefusal),
    Component(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
}
type Stop = WorldInterimCheckpointRefusal;

/// A weak installed owner port, separate from Query's passive owner custody.
#[derive(Debug, Clone)]
pub struct WorldInterimCheckpointPort {
    owner: Weak<super::RuntimeWorldOwnerRoot<(), (), (), (), ()>>,
}
impl super::RuntimeWorldOwner<(), (), (), (), ()> {
    pub fn interim_checkpoint_port(&self) -> WorldInterimCheckpointPort {
        WorldInterimCheckpointPort {
            owner: Arc::downgrade(&self.root),
        }
    }
}

#[derive(Debug)]
struct CapturedRow {
    meaning: Arc<MemoryPublicationMeaning>,
    before: CapturedNativeInterimComponent,
    after: CapturedNativeInterimComponent,
    bound_to_anchor: bool,
    digest: CanonicalDigestId,
}
#[derive(Debug)]
struct CapturedRegistry {
    coordinate: CanonicalDigestId,
    native_digest: CanonicalDigestId,
    covered: u64,
    transitions: u64,
    application: CanonicalDigestId,
    anchor: Arc<MemoryPublicationReceiverScope>,
    anchor_component: CapturedNativeInterimComponent,
    rows: ExecutionArray<CapturedRow>,
    bytes: ExecutionImmutableBytes,
    digest: CanonicalDigestId,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub struct CapturedWorldInterimAdmission(Arc<CapturedRegistry>);
#[derive(Clone, Debug)]
pub struct CapturedWorldInterimOrigin(Arc<Origin>);
#[derive(Debug)]
struct Origin {
    captured: Arc<CapturedRegistry>,
    wrapper_digest: CanonicalDigestId,
    wrapper: ExecutionImmutableBytes,
    section: Range<usize>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CapturedWorldInterimAdmission {
    pub fn registry_revision(&self) -> u64 {
        self.0.rows.len() as u64
    }
    pub fn body_digest(&self) -> CanonicalDigestId {
        self.0.digest
    }
    pub fn bytes(&self) -> &[u8] {
        self.0.bytes.bytes()
    }
    pub fn seal_transport(
        self,
        wrapper: ExecutionImmutableBytes,
        section: Range<usize>,
        native: &CapturedNativeInterimOrigin,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedWorldInterimOrigin, Stop> {
        request_scope::run(request, || {
            charge(request, self.0.bytes.len())?;
            if native.coordinate() != self.0.coordinate
                || wrapper.bytes().get(section.clone()) != Some(self.0.bytes.bytes())
            {
                return Err(Stop::BindingMismatch);
            }
            let cut = native.admission_cut();
            let scope_matches = match cut.scope() {
                worth_foundational::facade::CheckpointAdmissionScope::Interim {
                    transport_origin,
                    registry_revision,
                    world_snapshot_body_digest,
                } => {
                    transport_origin == self.0.coordinate.bytes()
                        && *registry_revision == self.registry_revision()
                        && world_snapshot_body_digest == self.0.digest.bytes()
                }
                worth_foundational::facade::CheckpointAdmissionScope::Home { .. } => false,
            };
            if !scope_matches
                || cut.canonical_through() != self.0.covered
                || cut.transitions_through() != self.0.transitions
                || cut.checkpoint_digest() != self.0.native_digest.bytes()
            {
                return Err(Stop::BindingMismatch);
            }
            native
                .validate_transport(wrapper.bytes(), request)
                .map_err(Stop::Native)?;
            let digest = native.wrapper_digest();
            let memory = reserve_arc::<Origin>(request, 0)?;
            Ok(CapturedWorldInterimOrigin(Arc::new(Origin {
                captured: self.0,
                wrapper_digest: digest,
                wrapper,
                section,
                _memory: memory,
            })))
        })
    }
}
impl CapturedWorldInterimOrigin {
    pub fn validate_transport(
        &self,
        bytes: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Stop> {
        request_scope::run(request, || {
            charge(request, 1)?;
            // Sealing authenticated this exact immutable allocation, retained
            // here for its full lifetime. Copied transports still hash below.
            if std::ptr::eq(bytes, self.0.wrapper.bytes()) {
                return Ok(());
            }
            charge(request, self.0.captured.bytes.len())?;
            if bytes.len() != self.0.wrapper.len()
                || wire::digest(bytes, request)? != self.0.wrapper_digest
                || bytes.get(self.0.section.clone()) != Some(self.0.captured.bytes.bytes())
            {
                return Err(Stop::AuthenticationMismatch);
            }
            Ok(())
        })
    }
}
pub(crate) fn charge(request: &ExecutionRequest<'_, '_>, work: usize) -> Result<(), Stop> {
    let work = u64::try_from(work).map_err(|_| Stop::SizeOverflow)?;
    request
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(work)
            } else {
                context.poll()
            }
        })
        .map_err(Stop::Request)?
        .0
        .map_err(|cause| Stop::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))
}
fn reserve_arc<T>(
    request: &ExecutionRequest<'_, '_>,
    extra: usize,
) -> Result<Option<ExecutionMemoryReservation>, Stop> {
    let layout = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| Stop::SizeOverflow)?
        .0
        .pad_to_align();
    let bytes = layout.size().checked_add(extra).ok_or(Stop::SizeOverflow)?;
    request
        .reserve_retained_memory(bytes as u64)
        .map_err(Stop::Request)
}
fn reference(
    observation: &ProductBranchObservation,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<MemoryPublicationReceiverScope>, Stop> {
    let name = observation.branch_identity().name().as_str();
    let memory = reserve_arc::<MemoryPublicationReceiverScope>(request, name.len())?;
    charge(request, name.len())?;
    let reference = MemoryProductBranchReference::from_observed(
        observation.owner_identity().get(),
        name.to_owned(),
        observation.lifecycle_incarnation().ordinal(),
        observation.reference_generation().get(),
        observation.selected_commit().ordinal(),
    )
    .map_err(|_| Stop::Malformed)?;
    Ok(Arc::new(MemoryPublicationReceiverScope {
        reference,
        _funding: memory,
    }))
}
