//! Explicit per-call custody; notification runs after the caller's outer guards.
use super::WorthQueryProductBranchOwnerCleanupDenial as Denial;
use super::{
    follower::WorthQueryProductRetentionFollower,
    registry::WorthQueryProductBranchOwnerCleanupRegistry,
};
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::branch::{DeletedRelationalBranch, RelationalBranchIdentity};
use {
    crate::domain_computation::primary_graph::application_contribution::execution_denial,
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial as Resource,
    crate::domain_computation::request_execution::WorthQueryAdvancementDenial as Advancement,
};

struct Retirement {
    follower: Arc<dyn WorthQueryProductRetentionFollower>,
    deleted: Option<DeletedRelationalBranch>,
    _receipt_memory: Option<ExecutionMemoryReservation>,
}
impl Drop for Retirement {
    fn drop(&mut self) {
        if let Some(deleted) = &self.deleted {
            self.follower.native_branch_deleted(deleted);
        }
    }
}
/// Declare before every guard that may enclose cleanup, and borrow through it.
/// The registry remains the sole Source owner until these actual receipts notify.
#[derive(Default)]
pub struct OwnerCleanupRetirements {
    entries: Vec<Retirement>,
    _buffer_memory: Option<ExecutionMemoryReservation>,
    // Always retain the upgraded follower, including admission/error paths.
    // Its own final destruction must also occur beyond the outer guards.
    follower: Option<Arc<dyn WorthQueryProductRetentionFollower>>,
}
impl OwnerCleanupRetirements {
    pub(super) fn bind(
        &mut self,
        registry: &WorthQueryProductBranchOwnerCleanupRegistry,
    ) -> Result<(), Denial> {
        let Some(weak) = registry.follower.get() else {
            return Ok(());
        };
        if let Some(existing) = &self.follower {
            if !Weak::ptr_eq(&Arc::downgrade(existing), weak) {
                return Err(Denial::CleanupUnavailable);
            }
        } else {
            self.follower = weak.upgrade();
        }
        Ok(())
    }
    pub(super) fn prepare(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        identity: &RelationalBranchIdentity,
    ) -> Result<Option<usize>, Denial> {
        let Some(follower) = &self.follower else {
            return Ok(None);
        };
        if !follower.manages_native_sources(identity) {
            return Ok(None);
        }
        let count = self.entries.len().checked_add(1).ok_or_else(overflow)?;
        let layout = std::alloc::Layout::array::<Retirement>(count).map_err(|_| overflow())?;
        let bytes = u64::try_from(layout.size()).map_err(|_| overflow())?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(request_stop)?;
        let receipt_memory = execution
            .reserve_retained_memory(identity.branch_id().0.len() as u64)
            .map_err(request_stop)?;
        let mut entries = Vec::new();
        entries.try_reserve_exact(count).map_err(|_| {
            Denial::Execution(Advancement::Resource(Resource::NativeAllocationRefused {
                kind: worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator,
                requested_payload_bytes: Some(bytes),
            }))
        })?;
        entries.append(&mut self.entries);
        entries.push(Retirement {
            follower: Arc::clone(follower),
            deleted: None,
            _receipt_memory: receipt_memory,
        });
        self.entries = entries;
        self._buffer_memory = memory;
        Ok(Some(count - 1))
    }
    pub(super) fn deleted(&mut self, slot: Option<usize>, deleted: DeletedRelationalBranch) {
        if let Some(slot) = slot {
            self.entries[slot].deleted = Some(deleted);
        }
    }
}
fn overflow() -> Denial {
    Denial::Execution(Advancement::Resource(Resource::CapacityOverflow))
}
fn request_stop(cause: worth_execution::WorkCeilingDenial) -> Denial {
    Denial::Execution(execution_denial::advancement_scope_stop(cause))
}
