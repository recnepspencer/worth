//! A protected source cut is data custody, never publication or Complete authority.
use super::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::{
    branch::RelationalBranchTransitionSequence, publication::patch::data::PatchStreamPosition,
};
use std::{
    any::Any,
    sync::{Arc, Weak},
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
mod refusal;
pub use refusal::RelationalDurableOwnedReadRefusal;
use RelationalDurableOwnedReadRefusal as Refusal;
/// Descriptive immutable bounds selected by the actual provider cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalDurableReadBounds {
    pub selected_checkpoint: bool,
    pub canonical: PatchStreamPosition,
    pub transitions: RelationalBranchTransitionSequence,
}
/// The provider's own original payload; no request or decoded authority is retained.
pub type RelationalDurableReadPayload = Box<dyn Any + Send + Sync>;
pub struct RelationalDurableReadProtection {
    issuer: Weak<dyn RelationalDurableColdSource>,
    bounds: RelationalDurableReadBounds,
    original: RelationalDurableReadPayload,
}
impl std::fmt::Debug for RelationalDurableReadProtection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalDurableReadProtection")
            .field("bounds", &self.bounds)
            .finish_non_exhaustive()
    }
}
impl RelationalDurableReadProtection {
    pub fn read_initial_inputs_owned(
        &self,
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Option<ExecutionImmutableBytes>, Refusal> {
        self.validate(source)?;
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        source.read_initial_inputs_owned(request, self.original.as_ref(), limit)
    }
    pub fn capture(
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let (bounds, original) = source.checkpoint_read_protection(request)?;
        Ok(Self {
            issuer: Arc::downgrade(source),
            bounds,
            original,
        })
    }
    pub fn bounds(&self) -> RelationalDurableReadBounds {
        self.bounds
    }
    pub(super) fn validate(
        &self,
        source: &Arc<dyn RelationalDurableColdSource>,
    ) -> Result<(), Refusal> {
        // The Weak keeps this allocation identity non-reusable. Never upgrade it.
        if !Weak::ptr_eq(&self.issuer, &Arc::downgrade(source)) {
            return Err(Refusal::ForeignSource);
        }
        Ok(())
    }
    pub fn read_commit_owned_at(
        &self,
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        self.validate(source)?;
        if position.0 == 0 || position > self.bounds.canonical {
            return Err(Refusal::Bounds);
        }
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        source.read_commit_owned(request, self.original.as_ref(), position, limit)
    }
    pub fn read_transition_owned_at(
        &self,
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
        sequence: RelationalBranchTransitionSequence,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        self.validate(source)?;
        if sequence.0 == 0 || sequence > self.bounds.transitions {
            return Err(Refusal::Bounds);
        }
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        source.read_transition_owned(request, self.original.as_ref(), sequence, limit)
    }
    pub fn read_member_owned_at(
        &self,
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        self.validate(source)?;
        if member.length > limit {
            return Err(Refusal::Storage(
                crate::storage::data::RelationalStorageUnavailableCause::LimitExceeded {
                    length: member.length,
                    limit,
                },
            ));
        }
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        source.read_member_owned(request, self.original.as_ref(), member, limit)
    }
}

#[cfg(test)]
mod tests;
