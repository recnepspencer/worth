//! Context metadata uses the common originally funded opaque custody owner.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::validation::candidate_source::opaque_custody::{
    FundedSourceMaterialReceipt, FundedSourceMaterialShare,
};
use std::any::Any;
use worth_execution::ExecutionRequest;
type Payload = dyn Any + Send + Sync;

/// Strong metadata with original allocation funding; no raw Arc, detachable
/// funding, READ or Source authority is exposed.
pub struct ManagedIndexSourceContextShare<T: ?Sized = Payload>(FundedSourceMaterialShare<T>);
impl<T: ?Sized> Clone for ManagedIndexSourceContextShare<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Any + Send + Sync> ManagedIndexSourceContextShare<T> {
    /// Refusal returns T for retirement outside runtime/registry guards.
    pub fn new(execution: &ExecutionRequest<'_, '_>, value: T) -> Result<Self, (Cause, T)> {
        FundedSourceMaterialShare::new(execution, value).map(Self)
    }
    pub fn receipt(&self) -> ManagedIndexSourceContextReceipt {
        ManagedIndexSourceContextReceipt(self.0.receipt())
    }
    pub fn erase(self) -> ManagedIndexSourceContextShare {
        ManagedIndexSourceContextShare(self.0.erase())
    }
}
impl<T: ?Sized> ManagedIndexSourceContextShare<T> {
    pub fn value(&self) -> &T {
        self.0.value()
    }
}
impl ManagedIndexSourceContextShare {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.value().downcast_ref()
    }
}

/// Weak originally funded context metadata; no context association is minted.
#[derive(Clone)]
pub struct ManagedIndexSourceContextReceipt(FundedSourceMaterialReceipt);
impl ManagedIndexSourceContextReceipt {
    pub fn upgrade(&self) -> Option<ManagedIndexSourceContextShare> {
        self.0.upgrade().map(ManagedIndexSourceContextShare)
    }
}
impl std::fmt::Debug for ManagedIndexSourceContextReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedIndexSourceContextReceipt")
            .finish_non_exhaustive()
    }
}
