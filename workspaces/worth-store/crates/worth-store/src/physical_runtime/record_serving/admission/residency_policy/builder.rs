use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::TransitionOutcome;

use super::{
    AdmittedPhysicalRecordResidencyPolicy, PhysicalOperationAllocationScope,
    PhysicalRecordResidencyPolicyOutcome, PhysicalSpeculativeWorkKind,
};
use crate::physical_runtime::record_serving::AdmittedPhysicalRecordFormat;

/// Starts a complete physical-memory envelope declaration for one Store.
///
/// The raw declaration grants no runtime authority. Call `builder`, declare
/// every required dimension, and then call `admit` against the admitted record
/// format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecordResidencyPolicy;

impl PhysicalRecordResidencyPolicy {
    /// Begins an incomplete residency-policy declaration.
    pub fn builder() -> PhysicalRecordResidencyPolicyBuilder {
        PhysicalRecordResidencyPolicyBuilder {
            declaration: worth_store_buffer_pool::PhysicalResidencyLimits::builder(),
        }
    }
}

/// An incomplete physical-memory envelope declaration.
///
/// Initialization and open accept only
/// `AdmittedPhysicalRecordResidencyPolicy`, never this builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecordResidencyPolicyBuilder {
    declaration: worth_store_buffer_pool::PhysicalResidencyLimitsBuilder,
}

impl PhysicalRecordResidencyPolicyBuilder {
    /// Sets the hard envelope for all live pool-owned bytes.
    pub const fn total_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.declaration = self.declaration.total_bytes(bytes);
        self
    }

    /// Sets the maximum bytes occupied by resident frame payloads.
    pub const fn resident_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.declaration = self.declaration.resident_bytes(bytes);
        self
    }

    /// Sets the maximum bytes occupied by frame-table metadata.
    pub const fn metadata_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.declaration = self.declaration.metadata_bytes(bytes);
        self
    }

    /// Sets the maximum number of frame identities.
    pub const fn frame_entries(mut self, entries: NonZeroU32) -> Self {
        self.declaration = self.declaration.frame_entries(entries);
        self
    }

    /// Sets the maximum number of simultaneously pinned frames.
    pub const fn pinned_frames(mut self, frames: NonZeroU32) -> Self {
        self.declaration = self.declaration.pinned_frames(frames);
        self
    }

    /// Sets the maximum number of live pin leases.
    pub const fn pin_leases(mut self, leases: NonZeroU32) -> Self {
        self.declaration = self.declaration.pin_leases(leases);
        self
    }

    /// Sets the maximum number of dirty frames.
    pub const fn dirty_frames(mut self, frames: NonZeroU32) -> Self {
        self.declaration = self.declaration.dirty_frames(frames);
        self
    }

    /// Sets the bytes reserved for replacing dirty frame contents.
    pub const fn dirty_replacement_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.declaration = self.declaration.dirty_replacement_bytes(bytes);
        self
    }

    /// Sets the aggregate ceiling for operation-owned temporary bytes.
    pub const fn operation_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.declaration = self.declaration.operation_bytes(bytes);
        self
    }

    /// Sets one operation scope's ceiling inside `operation_bytes`.
    pub const fn scope_bytes(
        mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: NonZeroU64,
    ) -> Self {
        self.declaration = self.declaration.scope_bytes(scope, bytes);
        self
    }

    /// Withholds bytes from ordinary scopes so maintenance can still allocate.
    pub const fn progress_headroom_bytes(mut self, bytes: u64) -> Self {
        self.declaration = self.declaration.progress_headroom_bytes(bytes);
        self
    }

    /// Sets one speculative work kind's frame ceiling.
    pub const fn speculative_frames(
        mut self,
        kind: PhysicalSpeculativeWorkKind,
        frames: NonZeroU32,
    ) -> Self {
        self.declaration = self.declaration.speculative_frames(kind, frames);
        self
    }

    /// Validates the complete declaration against the admitted page format.
    ///
    /// Success returns a sealed admitted policy. Denial names the missing or
    /// inconsistent dimension and constructs no buffer pool.
    pub fn admit(
        self,
        format: AdmittedPhysicalRecordFormat,
    ) -> PhysicalRecordResidencyPolicyOutcome {
        let page_bytes = NonZeroU64::new(u64::from(format.declaration().page_size().bytes()))
            .expect("an admitted physical format has a nonzero page size");
        match self.declaration.admit(page_bytes) {
            Ok(limits) => {
                TransitionOutcome::success(AdmittedPhysicalRecordResidencyPolicy { limits })
            }
            Err(denial) => TransitionOutcome::denied(denial.into()),
        }
    }
}
