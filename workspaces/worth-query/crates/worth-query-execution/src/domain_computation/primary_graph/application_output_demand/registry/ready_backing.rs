//! One prepared Ready cell, retained through every reader of its completion.

use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use super::{
    required_custody::RequiredOutputCustodyCapacity, DemandRegistryState,
    WorthQueryCompletedOutputDemand,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

struct ReadyCell {
    completion: OnceLock<WorthQueryCompletedOutputDemand>,
    _capacity: RequiredOutputCustodyCapacity,
}

/// Prepared before an output effect. It cannot be read as a Ready completion.
pub(in crate::domain_computation::primary_graph) struct PreparedReadyBacking {
    cell: Arc<ReadyCell>,
    producer_commit_authority:
        Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority>,
}

/// The completed cell pins the original authority and its refundable custody.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct ReadyCompletion(Arc<ReadyCell>);

/// The required custody one Ready row declares.
#[cfg(feature = "test-query-execution-observer")]
#[doc(hidden)]
pub fn required_ready_custody_bytes_for_test() -> usize {
    PreparedReadyBacking::retained_bytes()
}

impl PreparedReadyBacking {
    pub(super) const fn retained_bytes() -> usize {
        let header = 2 * std::mem::size_of::<usize>();
        let alignment = std::mem::align_of::<ReadyCell>();
        let body_offset = (header + alignment - 1) & !(alignment - 1);
        body_offset + std::mem::size_of::<ReadyCell>()
    }

    pub(super) fn new(capacity: RequiredOutputCustodyCapacity) -> Self {
        Self {
            cell: Arc::new(ReadyCell {
                completion: OnceLock::new(),
                _capacity: capacity,
            }),
            producer_commit_authority: None,
        }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn for_test() -> Self {
        Self::new(super::required_custody::test_capacity(
            Self::retained_bytes(),
        ))
    }

    pub(super) fn prepare(
        state: &DemandRegistryState,
        admission: &mut InvalidationEditAdmission,
        pending_indexed_bytes: usize,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let indexed = state
            .required_reserved_bytes
            .checked_add(pending_indexed_bytes)
            .and_then(|bytes| bytes.checked_add(Self::retained_bytes()))
            .ok_or_else(capacity_denial)?;
        if !state.has_required_capacity(indexed) {
            return Err(capacity_denial());
        }
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        admission
            .admit_read_scratch(Self::retained_bytes() as u64)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    work_denial()
                }
                _ => capacity_denial(),
            })?;
        let capacity = state.reserve_required_custody_capacity(Self::retained_bytes())?;
        Ok(Self::new(capacity))
    }

    /// Bind the mode selected for the actual producer execution before it
    /// publishes either a performed result or a Stable equality result.
    pub(in crate::domain_computation::primary_graph) fn bind_execution_mode(
        mut self,
        mode: crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority,
    ) -> Self {
        assert!(self.producer_commit_authority.is_none());
        self.producer_commit_authority = Some(mode);
        self
    }

    pub(in crate::domain_computation::primary_graph) fn complete(
        self,
        mut completion: WorthQueryCompletedOutputDemand,
    ) -> ReadyCompletion {
        completion.producer_commit_authority = self.producer_commit_authority;
        assert!(self.cell.completion.set(completion).is_ok());
        ReadyCompletion(self.cell)
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "Ready backing preparation exceeds request work",
    )
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "Ready backing preparation exceeds request memory",
    )
}

impl Deref for ReadyCompletion {
    type Target = WorthQueryCompletedOutputDemand;

    fn deref(&self) -> &Self::Target {
        self.0
            .completion
            .get()
            .expect("only a filled Ready cell is observable")
    }
}

impl ReadyCompletion {
    pub(in crate::domain_computation::primary_graph) fn same_cell(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[cfg(test)]
impl ReadyCompletion {
    pub(in crate::domain_computation::primary_graph) fn for_test(
        completion: WorthQueryCompletedOutputDemand,
    ) -> Self {
        PreparedReadyBacking::new(super::required_custody::test_capacity(
            PreparedReadyBacking::retained_bytes(),
        ))
        .complete(completion)
    }
}
