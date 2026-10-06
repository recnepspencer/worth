//! The state a partitioned computation keeps on the record its attempt
//! published, and the ledger custody that pays for it.
//!
//! A record holds the state only while the lineage ledger holds its bytes. A
//! state the ledger refuses is evicted: the record keeps only that it was.
//! An incremental run builds its state from its prior record's, so its
//! publication moves that record's reservation to the new record, but only
//! when that record still holds exactly the state the run built from.

use std::any::TypeId;
use std::sync::{Arc, OnceLock};

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::ProductBranchObservation;

use super::invalidation::InvalidationEditAdmission;
use super::recorded_output::RecordedOutput;
use super::retained_capacity::RetainedLineageCapacity;
use super::{
    ProductCoordinate, RecordedSettlementIdentity, SemanticSource,
    WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::application_contribution::{
    ComputationPrior, InstalledProducerEdition, RetainedComputation, SealedComputationRun,
    WorthQueryPartitionedComputationFullCause,
};

/// What a record holds of its computation's state.
pub(super) enum RecordedComputation {
    Retained {
        state: Arc<RetainedComputation>,
        capacity: RetainedLineageCapacity,
    },
    /// The ledger refused the state's bytes.
    Evicted,
}

/// The exact record a producer's run took its prior state from.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct PriorComputationRecord {
    cell: Arc<OnceLock<RecordedOutput>>,
}

impl PriorComputationRecord {
    /// Takes the reservation of the record's state when the record is the
    /// one being `displaced` and still holds exactly `cloned_from`. The
    /// record keeps no state after.
    fn take_capacity(
        &self,
        cloned_from: &Arc<RetainedComputation>,
        displaced: &Arc<RecordedSettlementIdentity>,
    ) -> Option<RetainedLineageCapacity> {
        let recorded = self.cell.get()?;
        if !Arc::ptr_eq(&recorded.settlement_identity, displaced) {
            return None;
        }
        let mut row = recorded
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let holds = matches!(
            &row.computation,
            Some(RecordedComputation::Retained { state, .. }) if Arc::ptr_eq(state, cloned_from)
        );
        if !holds {
            return None;
        }
        match row.computation.take() {
            Some(RecordedComputation::Retained { capacity, .. }) => Some(capacity),
            _ => None,
        }
    }
}

impl WorthQueryApplicationOutputLineage {
    /// What the live record at a demand's own address retained for the
    /// producer's next run: the latest record of the source partition in the
    /// observed occurrence, the one the demand's publication replaces. No
    /// ancestor occurrence is followed, so a run never takes the state of a
    /// record it does not replace. A stable alias is its own record here.
    /// Every lookup is charged to `admission` before it reads.
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
        let Some((generation, slot)) = self.partition_index.latest_admitted(
            &source,
            coordinate,
            source_partition_identity,
            admission,
        )?
        else {
            return Ok(ComputationPrior::new(
                edition,
                Err(WorthQueryPartitionedComputationFullCause::NoPriorRecord),
                None,
            ));
        };
        let cell = self.recorded_cell_at_partition_slot_admitted(
            &source,
            coordinate.occurrence,
            generation,
            source_partition_identity,
            slot,
            admission,
        )?;
        let retained = match &cell
            .get()
            .expect("a partition locator references a published record")
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .computation
        {
            Some(RecordedComputation::Retained { state, .. }) => Ok(Arc::clone(state)),
            Some(RecordedComputation::Evicted) => {
                Err(WorthQueryPartitionedComputationFullCause::Evicted)
            }
            None => Err(WorthQueryPartitionedComputationFullCause::NoPriorRecord),
        };
        Ok(ComputationPrior::new(
            edition,
            retained,
            Some(PriorComputationRecord { cell }),
        ))
    }

    /// Charges a sealed run's state on the ledger for the record about to
    /// publish it. The reservation moves only from the record this one
    /// displaces, and only when it is the prior the run built from and still
    /// holds that state; the difference is reserved or released. Any other
    /// run reserves afresh. A refusal evicts the state. The request memory
    /// the run's tree held is released only once the state is charged here
    /// or evicted, so the tree is never unheld while it lives.
    pub(super) fn retain_computation(
        &self,
        sealed: SealedComputationRun,
        prior: Option<&PriorComputationRecord>,
        displaced: Option<&Arc<RecordedSettlementIdentity>>,
    ) -> RecordedComputation {
        let SealedComputationRun {
            state,
            cloned_from,
            tree_memory,
        } = sealed;
        let moved = prior.zip(cloned_from.as_ref()).zip(displaced).and_then(
            |((prior, cloned_from), displaced)| prior.take_capacity(cloned_from, displaced),
        );
        drop(cloned_from);
        let Some(bytes) = state.retained_bytes() else {
            drop(state);
            return RecordedComputation::Evicted;
        };
        let capacity = match moved {
            Some(mut capacity) => {
                let held = capacity.bytes();
                if bytes >= held {
                    capacity.reserve_additional(bytes - held).map(|()| capacity)
                } else {
                    capacity.release_part(held - bytes);
                    Ok(capacity)
                }
            }
            None => self.retention.reserve(bytes),
        };
        let recorded = match capacity {
            Ok(capacity) => RecordedComputation::Retained {
                state: Arc::new(state),
                capacity,
            },
            Err(_) => {
                drop(state);
                RecordedComputation::Evicted
            }
        };
        drop(tree_memory);
        recorded
    }
}
