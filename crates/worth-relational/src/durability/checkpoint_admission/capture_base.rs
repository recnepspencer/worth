//! Selected capture DATA shares its original bills; it contains no runtime edge.
use super::{
    RelationalCheckpointAdmissionArtifact, RelationalCheckpointAdmissionRefusal as Refusal,
};
use crate::durability::{
    data::RelationalCheckpointDirectory,
    source_initial_inputs::{
        normalized::NormalizedOriginalInputs, RelationalCompleteSourceArtifact,
    },
};
use std::sync::{Arc, OnceLock};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub(crate) struct SelectedCheckpointCaptureBase {
    source: RelationalCompleteSourceArtifact,
    image: RelationalCheckpointAdmissionArtifact,
    normalized: Arc<NormalizedOriginalInputs>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone)]
pub(crate) struct FrozenSelectedCaptureBase {
    pub(crate) directory: Arc<RelationalCheckpointDirectory>,
    pub(crate) state: Arc<SelectedCheckpointCaptureBase>,
}

/// Shared only by clones of one prepared publication, before its actual ACK.
#[derive(Debug)]
pub(crate) struct PreparedCheckpointCaptureBase {
    value: OnceLock<Arc<SelectedCheckpointCaptureBase>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl worth_proof::AuthorityMarker for SelectedCheckpointCaptureBase {}
impl SelectedCheckpointCaptureBase {
    pub(crate) fn source(&self) -> &RelationalCompleteSourceArtifact {
        &self.source
    }
    pub(crate) fn image(&self) -> &RelationalCheckpointAdmissionArtifact {
        &self.image
    }
    pub(crate) fn normalized(&self) -> &NormalizedOriginalInputs {
        &self.normalized
    }
    pub(crate) fn restore_original(
        runtime: &crate::runtime::RelationalRuntime,
        original: &crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
        match (&original.selected, &original.normalized) {
            (None, None) => Ok(()),
            (Some(_), Some(_)) => runtime
                .durability
                .checkpoint_selection()
                .restore_capture_base(original, request)
                .map_err(crate::durability::data::RelationalRecoveryStop::AdmissionImage),
            (None, Some(_)) | (Some(_), None) => {
                Err(crate::durability::checkpoints::region_directory::corrupt(
                    "selected original capture base is incomplete",
                )
                .into())
            }
        }
    }
    pub(crate) fn from_bound_inputs(
        owner: &super::CheckpointAdmissionBinding,
        original: &crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::RelationalReadmitted<Self>, Refusal> {
        let directory = owner.capture_directory()?;
        let source = original.selected.as_ref().ok_or(Refusal::MissingArtifact)?;
        let image = directory
            .admission_image
            .as_ref()
            .ok_or(Refusal::MissingArtifact)?;
        let (origin, digest) = match image.cut().scope() {
            worth_foundational::facade::CheckpointAdmissionScope::Home {
                native_origin,
                complete_source_digest,
                ..
            } => (native_origin, complete_source_digest),
            worth_foundational::facade::CheckpointAdmissionScope::Interim { .. } => {
                return Err(Refusal::BindingMismatch(
                    super::CheckpointAdmissionBindingMismatch::Scope,
                ))
            }
        };
        if source.covered() != directory.manifest.covered_position
            || source.covered_transitions() != directory.manifest.coverage.up_to_branch_transition
            || source.covered().0 != image.cut().canonical_through()
            || source.covered_transitions().0 != image.cut().transitions_through()
            || origin != &source.initial_coordinate().1
            || digest != &super::binding_digest(source.as_bytes(), request)?
            || image.cut().checkpoint_digest() != &super::binding_digest(source.checkpoint().map_err(|cause| match cause {
                crate::durability::source_initial_inputs::RelationalInitialInputsRefusal::Request(cause) => Refusal::Request(cause),
                _ => Refusal::InvalidProjectedMeaning,
            })?.bytes(), request)? {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        // Derive the projection from the SAME bound immutable source; a parsed
        // projection supplied beside it cannot establish carried authority.
        let normalized = crate::durability::source_initial_inputs::normalized_admission::readmit(
            source.normalized_bytes(),
            source.covered().0,
            source.covered_transitions().0,
            request,
        )
        .map_err(|cause| match cause {
            crate::durability::source_initial_inputs::RelationalInitialInputsRefusal::Request(
                cause,
            ) => Refusal::Request(cause),
            _ => Refusal::InvalidProjectedMeaning,
        })?;
        Ok(worth_proof::RelationalReadmitted::from_owner(Self {
            source: source.clone(),
            image: image.clone(),
            normalized,
            _memory: reserve::<Self>(request)?,
        }))
    }
}
impl PreparedCheckpointCaptureBase {
    pub(crate) fn allocate(request: &ExecutionRequest<'_, '_>) -> Result<Arc<Self>, Refusal> {
        Ok(Arc::new(Self {
            value: OnceLock::new(),
            _memory: reserve::<Self>(request)?,
        }))
    }
    pub(in crate::durability) fn install(
        self: &Arc<Self>,
        sealed: worth_proof::NativeCheckpointOriginalsVerified<crate::durability::authority::checkpoint_complete::admission_projection::SealedCheckpointCaptureData>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let (publication, source, image, normalized) = sealed.consume(|sealed| sealed.into_parts());
        if !Arc::ptr_eq(self, &publication) {
            return Err(Refusal::OwnerUnavailable);
        }
        let base = Arc::new(SelectedCheckpointCaptureBase {
            source,
            image,
            normalized,
            _memory: reserve::<SelectedCheckpointCaptureBase>(request)?,
        });
        match self.value.set(base) {
            Ok(()) => Ok(()),
            Err(next)
                if self
                    .value
                    .get()
                    .is_some_and(|prior| prior.image.as_bytes() == next.image.as_bytes()) =>
            {
                Ok(())
            }
            Err(_) => Err(Refusal::OwnerUnavailable),
        }
    }
    pub(crate) fn snapshot(&self) -> Option<Arc<SelectedCheckpointCaptureBase>> {
        self.value.get().cloned()
    }
}
fn reserve<T>(
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, Refusal> {
    let size = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| Refusal::CapacityOverflow)?
        .0
        .pad_to_align()
        .size();
    request
        .reserve_retained_memory(size as u64)
        .map_err(|cause| Refusal::Request(cause.into()))
}
