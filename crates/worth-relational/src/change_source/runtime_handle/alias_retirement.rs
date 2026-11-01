//! Relinquish a local immutable alias while its authentic owner remains alive.
use super::{RelationalRuntimeHandle, RuntimeOwnership};
use crate::runtime::RelationalRuntime;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRuntimeHandleAliasRetirementDenial {
    SharedRepresentation,
    NotOwner,
    ForeignOwner,
    Execution(WorkCeilingDenial),
}

/// Original passive metadata after the local runtime Arc was discharged.
/// It grants no runtime access and proves no Native business completion.
#[must_use]
pub struct RetiredRelationalRuntimeHandleAlias {
    runtime_instance_id: u64,
    _lock_observation: Arc<super::lock_observation::LockObservationSlot>,
}

impl RelationalRuntimeHandle {
    /// Pure owner/representation validation, without admission, locks or callbacks.
    pub fn validate_immutable_owner_alias(
        &self,
        owner: &RelationalRuntime,
    ) -> Result<(), RelationalRuntimeHandleAliasRetirementDenial> {
        let RuntimeOwnership::Immutable(runtime) = &self.ownership else {
            return Err(RelationalRuntimeHandleAliasRetirementDenial::SharedRepresentation);
        };
        if !owner.is_owner() {
            return Err(RelationalRuntimeHandleAliasRetirementDenial::NotOwner);
        }
        if !std::ptr::eq(runtime.as_ref(), owner) {
            return Err(RelationalRuntimeHandleAliasRetirementDenial::ForeignOwner);
        }
        Ok(())
    }

    /// Consume this alias without stopping its owner. Every refusal returns the
    /// same original handle. The borrowed matching owner prevents final Arc Drop.
    pub fn try_retire_immutable_alias(
        self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<
        RetiredRelationalRuntimeHandleAlias,
        (Self, RelationalRuntimeHandleAliasRetirementDenial),
    > {
        if let Err(cause) = self.validate_immutable_owner_alias(owner) {
            return Err((self, cause));
        }
        if let Err(cause) = request.consult() {
            return Err((
                self,
                RelationalRuntimeHandleAliasRetirementDenial::Execution(cause),
            ));
        }
        let Self {
            ownership,
            runtime_instance_id,
            lock_observation,
        } = self;
        match ownership {
            RuntimeOwnership::Immutable(runtime) => {
                // Safe borrowing forbids moving the only owning Arc while its
                // pointee is borrowed. Exact identity above therefore proves a
                // separate live owner keeps this nonfinal discharge from Drop.
                drop(runtime);
                Ok(RetiredRelationalRuntimeHandleAlias {
                    runtime_instance_id,
                    _lock_observation: lock_observation,
                })
            }
            ownership @ RuntimeOwnership::Shared(_) => Err((
                Self {
                    ownership,
                    runtime_instance_id,
                    lock_observation,
                },
                RelationalRuntimeHandleAliasRetirementDenial::SharedRepresentation,
            )),
        }
    }
}

impl RetiredRelationalRuntimeHandleAlias {
    /// Reclaim the original slot only outside request-controlled work.
    pub fn reclaim(self) {
        drop(self);
    }
}

impl std::fmt::Debug for RetiredRelationalRuntimeHandleAlias {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RetiredRelationalRuntimeHandleAlias")
            .field("runtime_instance_id", &self.runtime_instance_id)
            .finish_non_exhaustive()
    }
}
