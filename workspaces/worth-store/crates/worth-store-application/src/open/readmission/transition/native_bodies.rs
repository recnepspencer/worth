//! The original retained body roster shares its backing with every plan holder.
use super::ApplicationHomeTransitionIntentDenial as Denial;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_foundational::facade::HomeProgramTransitionBodyDigest;

#[derive(Debug)]
pub(super) struct HomeNativeStepBodies {
    values: ExecutionArray<HomeProgramTransitionBodyDigest>,
    _sharing: Option<ExecutionMemoryReservation>,
}
impl HomeNativeStepBodies {
    pub(super) fn capture(
        values: &[HomeProgramTransitionBodyDigest],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let mut array = ExecutionRequestArrayBuilder::allocate(*request, values.len())
            .map_err(Denial::Backing)?;
        for value in values {
            request.consult().map_err(Denial::Execution)?;
            array.push(*value).map_err(Denial::Backing)?;
        }
        Self::retain(array.seal().map_err(Denial::Backing)?, request)
    }
    pub(super) fn retain(
        values: ExecutionArray<HomeProgramTransitionBodyDigest>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| Denial::SizeLimit)?
            .0
            .pad_to_align();
        let sharing = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(Denial::Execution)?;
        Ok(Arc::new(Self {
            values,
            _sharing: sharing,
        }))
    }
    pub(super) fn values(&self) -> &[HomeProgramTransitionBodyDigest] {
        &self.values
    }
}
