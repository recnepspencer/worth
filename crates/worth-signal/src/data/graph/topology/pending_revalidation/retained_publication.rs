//! Accounted retained replacement of the pending-revalidation waiter index.

use std::sync::Arc;

use super::{PreparedPendingRevalidationIndex, SignalGraph};
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::{
    map_node_edit_accounting as map_accounting, map_node_edit_retention as map_retention,
};
use crate::data::handle::NodeId;
use crate::data::persistent_ord_map::{
    PersistentOrdMap, RetainedMapMutationDenial, RetainedMapMutationOutcome,
};
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
};

#[derive(Debug)]
pub(crate) struct PreparedRetainedPendingRevalidationIndex {
    waiters: PersistentOrdMap<NodeId, im::OrdSet<NodeId>>,
    resources: SignalConditionalRetentionReservation,
}

impl PreparedPendingRevalidationIndex {
    pub(crate) fn prepare_retained(
        self,
        graph: &mut SignalGraph,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        maximum: Charge,
        work: &mut Work,
    ) -> Result<PreparedRetainedPendingRevalidationIndex, SignalError> {
        let source_growth = graph
            .topology
            .pending_revalidation_waiters
            .prepare_fork_growth(work)
            .map_err(map_fork_growth)?;
        let current = graph
            .topology
            .pending_revalidation_waiters
            .prepared_retained_charge()
            .map_err(map_mutation)?;
        if current > maximum {
            return Err(SignalError::EvaluationStorageCapacityExhausted);
        }
        let initial = current
            .checked_add(source_growth)
            .and_then(|charge| {
                charge.checked_add(arc_allocation_charge::<
                    SignalConditionalRetentionReservation,
                >()?)
            })
            .map_err(map_accounting)?;
        let mut resources = ledger.reserve(0, initial).map_err(map_retention)?;
        let mut waiters = graph
            .topology
            .pending_revalidation_waiters
            .fork_reserved(&mut resources);
        let mut admitted = current;

        for (producer, next) in self.buckets {
            if next.is_empty() && !waiters.contains_key(&producer) {
                continue;
            }
            let peak = waiters
                .prepare_insert_staging_charge(&producer, &next, work)
                .map_err(map_mutation)?;
            if peak > maximum {
                return Err(SignalError::EvaluationStorageCapacityExhausted);
            }
            if peak > admitted {
                resources
                    .grow(peak.checked_sub(admitted).map_err(map_accounting)?)
                    .map_err(map_retention)?;
                admitted = peak;
            }
            let outcome = if next.is_empty() {
                waiters
                    .remove_with_retained_charge(&producer, work)
                    .map_err(map_mutation)?
            } else {
                waiters
                    .insert_with_retained_charge(producer, next, work)
                    .map_err(map_mutation)?
            };
            accounted(outcome)?;
        }

        let retained = waiters.prepared_retained_charge().map_err(map_mutation)?;
        if retained > maximum {
            return Err(SignalError::EvaluationStorageCapacityExhausted);
        }
        let payload = retained
            .checked_add(
                arc_allocation_charge::<SignalConditionalRetentionReservation>()
                    .map_err(map_accounting)?,
            )
            .map_err(map_accounting)?;
        resources
            .shrink_payload_to(payload)
            .map_err(map_retention)?;
        Ok(PreparedRetainedPendingRevalidationIndex { waiters, resources })
    }
}

impl PreparedRetainedPendingRevalidationIndex {
    pub(crate) fn install(self, graph: &mut SignalGraph) {
        let old_waiters = std::mem::replace(
            &mut graph.topology.pending_revalidation_waiters,
            self.waiters,
        );
        let old_custody = graph
            .topology
            .pending_revalidation_storage_custody
            .replace(Arc::new(self.resources));
        drop(old_waiters);
        drop(old_custody);
    }
}

fn accounted<R>(outcome: RetainedMapMutationOutcome<R>) -> Result<R, SignalError> {
    match outcome {
        RetainedMapMutationOutcome::Accounted { output, .. } => Ok(output),
        RetainedMapMutationOutcome::Unaccounted { denial, .. } => Err(map_accounting(denial)),
    }
}

fn map_mutation(denial: RetainedMapMutationDenial) -> SignalError {
    match denial {
        RetainedMapMutationDenial::Accounting(denial) => map_accounting(denial),
        RetainedMapMutationDenial::PreparationRequired | RetainedMapMutationDenial::MissingKey => {
            SignalError::EvaluationStorageUnavailable
        }
    }
}

fn map_fork_growth(denial: RetainedStorageForkGrowthDenial) -> SignalError {
    match denial {
        RetainedStorageForkGrowthDenial::Accounting(denial) => map_accounting(denial),
        RetainedStorageForkGrowthDenial::PreparationRequired => {
            SignalError::EvaluationStorageUnavailable
        }
    }
}
