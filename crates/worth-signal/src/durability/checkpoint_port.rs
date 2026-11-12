//! A weak owner-issued checkpoint door, without a second owner or caller callback.
use super::{SignalAuthoredHomeImage, SignalDurableOwner};
use crate::branch::owner_services::durable_publication::{
    SignalDurableAdvanceDenial, SignalDurablePublicationState,
};
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
trait SignalAuthoredCheckpointService: Send + Sync {
    fn capture(
        self: Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalAuthoredHomeImage, SignalDurableAdvanceDenial>;
}
impl<D, I, T> SignalAuthoredCheckpointService for SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn capture(
        self: Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalAuthoredHomeImage, SignalDurableAdvanceDenial> {
        self.capture_authored_home_image(execution)
    }
}
#[derive(Clone)]
pub struct SignalAuthoredCheckpointPort {
    owner: Weak<dyn SignalAuthoredCheckpointService>,
}
impl std::fmt::Debug for SignalAuthoredCheckpointPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalAuthoredCheckpointPort")
            .finish_non_exhaustive()
    }
}
impl SignalAuthoredCheckpointPort {
    pub fn capture(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalAuthoredHomeImage, SignalDurableAdvanceDenial> {
        self.owner
            .upgrade()
            .ok_or(SignalDurableAdvanceDenial::OwnerUnavailable)?
            .capture(execution)
    }
}
impl<D, I, E, Ctx, T> SignalDurableOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn checkpoint_port(&self) -> SignalAuthoredCheckpointPort {
        let owner: Arc<dyn SignalAuthoredCheckpointService> = self.publication.clone();
        SignalAuthoredCheckpointPort {
            owner: Arc::downgrade(&owner),
        }
    }
}
