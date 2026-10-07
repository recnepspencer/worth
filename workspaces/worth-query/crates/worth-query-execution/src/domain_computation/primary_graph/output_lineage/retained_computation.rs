//! The state a partitioned computation keeps on the record its attempt
//! published, and the ledger custody that pays for it.
//!
//! Every holder retains the same state and ledger ticket through one Arc.
//! A sole displaced state can yield its ticket only after its final prior is
//! consumed. A fork horizon or another holder preserves the old custody and
//! the successor reserves its distinct state in full. Refusal evicts only
//! the incoming state.

use std::any::TypeId;
use std::sync::{Arc, OnceLock};

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::ProductBranchObservation;

use super::invalidation::InvalidationEditAdmission;
use super::recorded_output::RecordedOutput;
use super::retained_capacity::RetainedLineageCapacity;
use super::CustodiedComputation;
use super::{
    ProductCoordinate, RecordedSettlementIdentity, SemanticSource,
    WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::application_contribution::{
    ComputationPrior, InstalledProducerEdition, PriorAbsence, SealedComputationRun,
};

/// What a record holds of its computation's state.
pub(super) enum RecordedComputation {
    Retained(Arc<CustodiedComputation>),
    Absent(PriorAbsence),
}

/// The exact record a producer's run took its prior state from.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct PriorComputationRecord {
    cell: Arc<OnceLock<RecordedOutput>>,
}

impl PriorComputationRecord {
    #[cfg(test)]
    pub(super) fn from_recorded_cell_for_test(cell: Arc<OnceLock<RecordedOutput>>) -> Self {
        assert!(
            cell.get().is_some(),
            "the test prior has a real published record"
        );
        Self { cell }
    }

    /// The sealed prior is consumed before exclusivity is established. A fork
    /// horizon or another holder leaves the existing state and ticket intact.
    fn take_capacity(
        &self,
        cloned_from: Arc<CustodiedComputation>,
        displaced: &Arc<RecordedSettlementIdentity>,
        lineage: &WorthQueryApplicationOutputLineage,
        bytes: u64,
        prepaid_fork_scan: usize,
    ) -> Result<Option<RetainedLineageCapacity>, super::super::WorthQueryOutputDemandDenial> {
        let Some(recorded) = self.cell.get() else {
            return Ok(None);
        };
        if !Arc::ptr_eq(&recorded.settlement_identity, displaced)
            || lineage.origins.len() > prepaid_fork_scan
            || lineage.fork_pins_computation(recorded)
        {
            return Ok(None);
        }
        let mut row = recorded
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let exclusive = matches!(&row.computation, RecordedComputation::Retained(handle)
            if Arc::ptr_eq(handle, &cloned_from) && Arc::strong_count(handle) == 2);
        if !exclusive {
            return Ok(None);
        }
        drop(cloned_from);
        let RecordedComputation::Retained(handle) = &mut row.computation else {
            unreachable!();
        };
        // Refusal preserves the sole prior record and its original ticket.
        let Some(owned) = Arc::get_mut(handle) else {
            return Ok(None);
        };
        owned.admit_successor_growth(bytes)?;
        let held = std::mem::replace(
            &mut row.computation,
            RecordedComputation::Absent(PriorAbsence::Moved),
        );
        match held {
            RecordedComputation::Retained(handle) => match Arc::try_unwrap(handle) {
                Ok(owned) => Ok(Some(owned.into_capacity())),
                Err(_) => unreachable!(
                    "the row lock and consumed sealed prior establish exclusive custody"
                ),
            },
            RecordedComputation::Absent(_) => {
                unreachable!("the pointer check established retained custody")
            }
        }
    }
}

impl WorthQueryApplicationOutputLineage {
    /// The latest local record is authoritative, including a typed absence.
    /// With no local record, follow exact captured fork horizons recursively.
    /// Sharing supplies a prior only; the ordinary basis and snapshot checks
    /// still establish whether its calls can be carried. Every lookup is
    /// charged to `admission` before it reads.
    pub(in crate::domain_computation::primary_graph) fn computation_prior_at_address<
        Binding: 'static,
    >(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &ProductBranchObservation,
        source_partition_identity: [u8; 32],
        edition: InstalledProducerEdition,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ComputationPrior, CompanionPreflightStop> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        let coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        let Some(cell) = self.computation_cell_in_ancestry(
            &source,
            coordinate,
            source_partition_identity,
            admission,
        )?
        else {
            return Ok(ComputationPrior::new(
                edition,
                Err(PriorAbsence::FirstRun.full_cause()),
                None,
            ));
        };
        let retained = match &cell
            .get()
            .expect("a partition locator references a published record")
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .computation
        {
            RecordedComputation::Retained(handle) => Ok(Arc::clone(handle)),
            RecordedComputation::Absent(absence) => Err(absence.full_cause()),
        };
        Ok(ComputationPrior::new(
            edition,
            retained,
            Some(PriorComputationRecord { cell }),
        ))
    }

    /// A sole, unpinned, exact displaced prior transfers its ticket after
    /// growth is admitted and the old state is consumed. Shared and pinned
    /// priors preserve custody; their successor is charged in full. Refusal
    /// or an unmeasured incoming state leaves existing holders intact. Run
    /// memory remains held until the incoming state is charged or discarded.
    pub(super) fn retain_computation(
        &self,
        sealed: SealedComputationRun,
        prior: Option<&PriorComputationRecord>,
        displaced: Option<&Arc<RecordedSettlementIdentity>>,
        prepaid_fork_scan: usize,
    ) -> RecordedComputation {
        let SealedComputationRun {
            state,
            cloned_from,
            tree_memory,
        } = sealed;
        let Some(bytes) = CustodiedComputation::retained_bytes_for(&state) else {
            drop(state);
            return RecordedComputation::Absent(PriorAbsence::Unmeasured);
        };
        let moved = prior.zip(cloned_from).zip(displaced).map_or(
            Ok(None),
            |((prior, cloned_from), displaced)| {
                prior.take_capacity(cloned_from, displaced, self, bytes, prepaid_fork_scan)
            },
        );
        let capacity = match moved {
            Ok(Some(mut capacity)) => {
                let held = capacity.bytes();
                if bytes < held {
                    capacity.release_part(held - bytes);
                }
                Ok(capacity)
            }
            Ok(None) => self.retention.reserve(bytes),
            Err(refusal) => Err(refusal),
        };
        let recorded = match capacity {
            Ok(capacity) => {
                RecordedComputation::Retained(CustodiedComputation::new(state, capacity))
            }
            Err(_refusal) => {
                drop(state);
                RecordedComputation::Absent(PriorAbsence::Evicted)
            }
        };
        drop(tree_memory);
        recorded
    }
}

impl RecordedComputation {
    pub(super) fn shared(&self) -> Self {
        match self {
            Self::Retained(handle) => Self::Retained(Arc::clone(handle)),
            Self::Absent(reason) => Self::Absent(reason.clone()),
        }
    }
}

mod ancestry;

#[cfg(test)]
mod tests;
