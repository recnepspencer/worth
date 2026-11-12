//! Fixed original-request storage owns each genuine, move-only Native effect.
use crate::durability::{WorldDurableMediumDenial as Medium, WorldReadmissionDenial as Denial};
use std::sync::Mutex;
use worth_execution::{
    ExecutionArray, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity as Attempt;
use worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect as Effect;

pub(super) struct ReadmittedNativeEffects {
    indices: ExecutionOrderedMap<Attempt, usize>,
    slots: ExecutionArray<Mutex<Option<Effect>>>,
}
pub(super) struct NativeEffectAdmission<'request, 'authority> {
    indices: ExecutionOrderedMap<Attempt, usize>,
    slots: ExecutionRequestArrayBuilder<'request, 'authority, Mutex<Option<Effect>>>,
}
impl<'request, 'authority> NativeEffectAdmission<'request, 'authority> {
    pub(super) fn allocate(
        request: ExecutionRequest<'request, 'authority>,
        count: usize,
    ) -> Result<Self, Denial> {
        Ok(Self {
            indices: Default::default(),
            slots: ExecutionRequestArrayBuilder::allocate(request, count)
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?,
        })
    }
    pub(super) fn contains(&self, attempt: &Attempt) -> bool {
        self.indices.contains_key(attempt)
    }
    pub(super) fn push(
        &mut self,
        attempt: &Attempt,
        effect: Effect,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let index = self.slots.len();
        self.indices
            .insert_cloned_native_for_request(attempt, &index, *request)
            .map_err(|cause| Denial::Medium(Medium::Ordered(cause)))?;
        self.slots
            .push(Mutex::new(Some(effect)))
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))
    }
    pub(super) fn seal(self) -> Result<ReadmittedNativeEffects, Denial> {
        Ok(ReadmittedNativeEffects {
            indices: self.indices,
            slots: self
                .slots
                .seal()
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?,
        })
    }
}
impl ReadmittedNativeEffects {
    pub(super) fn take(&self, attempt: &Attempt) -> Option<Effect> {
        self.slots[*self.indices.get(attempt)?]
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
    }
}
