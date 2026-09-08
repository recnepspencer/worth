use super::PhysicalIntegrityScrubTarget;
use std::time::Duration;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

pub(super) const MAX_TARGETS: usize = 4096;
const MAX_WINDOW_BYTES: u32 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityScrubRequestDenial {
    EmptyScope,
    TargetLimitExceeded,
    TargetScopeMismatch,
    DuplicateOrOverlappingTarget,
    WindowBoundExceeded,
    TotalByteBoundExceeded,
    InvalidDeadline,
    RuntimeScopeMismatch,
    ActiveHandleLimitExceeded,
    RuntimeClosed,
}

/// Bounded diagnostic intent. Completion covers exactly these targets, never
/// implies a complete-store traversal and never authorizes a repair.
#[derive(Debug)]
pub struct ManagedPhysicalIntegrityScrubRequest {
    pub(in crate::physical_runtime) store: StableStoreIdentity,
    pub(super) targets: Box<[PhysicalIntegrityScrubTarget]>,
    pub(super) deadline: Duration,
}

impl ManagedPhysicalIntegrityScrubRequest {
    pub fn new(
        store: StableStoreIdentity,
        targets: impl IntoIterator<Item = PhysicalIntegrityScrubTarget>,
        maximum_window_bytes: u32,
        maximum_total_bytes: u64,
        deadline: Duration,
    ) -> Result<Self, PhysicalIntegrityScrubRequestDenial> {
        use PhysicalIntegrityScrubRequestDenial as Denial;
        if maximum_window_bytes == 0 || maximum_window_bytes > MAX_WINDOW_BYTES {
            return Err(Denial::WindowBoundExceeded);
        }
        if deadline.is_zero() || deadline > Duration::from_secs(24 * 60 * 60) {
            return Err(Denial::InvalidDeadline);
        }
        let mut exact = Vec::new();
        let mut bytes = 0_u64;
        for target in targets {
            if exact.len() == MAX_TARGETS {
                return Err(Denial::TargetLimitExceeded);
            }
            if target.scope().store_identity() != store {
                return Err(Denial::RuntimeScopeMismatch);
            }
            if target.range().length() > maximum_window_bytes {
                return Err(Denial::WindowBoundExceeded);
            }
            bytes = bytes
                .checked_add(target.range().length() as u64)
                .filter(|total| *total <= maximum_total_bytes)
                .ok_or(Denial::TotalByteBoundExceeded)?;
            if exact
                .iter()
                .any(|prior: &PhysicalIntegrityScrubTarget| prior.range().overlaps(target.range()))
            {
                return Err(Denial::DuplicateOrOverlappingTarget);
            }
            exact.push(target);
        }
        if exact.is_empty() {
            return Err(Denial::EmptyScope);
        }
        Ok(Self {
            store,
            targets: exact.into_boxed_slice(),
            deadline,
        })
    }
    pub fn targets(&self) -> &[PhysicalIntegrityScrubTarget] {
        &self.targets
    }
}
