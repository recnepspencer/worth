//! One prepared Ready cell, retained through every reader of its completion.
//!
//! The cell also keeps the mode a required wave executes the output's producer
//! in again. An executed output has the mode that executed it. A restored
//! output was executed by no demand of this runtime: it takes the mode of the
//! first demand that advances it.

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

type ProducerMode =
    crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority;

struct ReadyCell {
    completion: OnceLock<WorthQueryCompletedOutputDemand>,
    producer_mode: OnceLock<ProducerMode>,
    _capacity: RequiredOutputCustodyCapacity,
}

/// Prepared before an output effect. It cannot be read as a Ready completion.
pub(in crate::domain_computation::primary_graph) struct PreparedReadyBacking {
    cell: Arc<ReadyCell>,
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

#[cfg(feature = "test-query-execution-observer")]
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// The required-retained bytes this runtime's demand registry holds now:
    /// required members and the custody beneath them.
    #[doc(hidden)]
    pub fn required_custody_bytes_for_test(&self) -> usize {
        let state = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.required_reserved_bytes
            + state
                .required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire)
    }
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
                producer_mode: OnceLock::new(),
                _capacity: capacity,
            }),
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
            return Err(super::required_custody::full_custody_denial(
                Self::retained_bytes(),
                state.required_budget_bytes,
            ));
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
        self,
        mode: ProducerMode,
    ) -> Self {
        assert!(self.cell.producer_mode.set(mode).is_ok());
        self
    }

    pub(in crate::domain_computation::primary_graph) fn complete(
        self,
        completion: WorthQueryCompletedOutputDemand,
    ) -> ReadyCompletion {
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

    /// The mode a required wave executes this output's producer in again.
    /// A restored output has none until a demand advances it.
    pub(in crate::domain_computation::primary_graph) fn producer_mode(
        &self,
    ) -> Option<&ProducerMode> {
        self.0.producer_mode.get()
    }

    /// The demand advancing this output does so in `mode`. A restored output
    /// keeps the first such mode; an executed one keeps its own.
    pub(in crate::domain_computation::primary_graph) fn advanced_in(&self, mode: &ProducerMode) {
        if self.0.producer_mode.get().is_none() {
            drop(self.0.producer_mode.set(mode.clone()));
        }
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
