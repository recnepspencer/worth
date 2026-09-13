use crate::data::graph::storage::execution_basis::{
    SignalExecutionBasis, SignalExecutionBasisChargeDenial,
};
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation, RetainedStoragePreparationDenial,
    SignalConditionalRetentionDenial, SignalConditionalRetentionLedger,
    SignalConditionalRetentionReservation,
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalConditionalBasisCaptureDenial {
    CapacityExhausted,
    WorkExhausted { maximum_visits: usize },
    OwnerUnavailable,
    Unavailable,
}

/// One exact captured basis and its complete retained representation custody.
/// Source conversion reservations live inside the backing allocations instead.
pub(crate) struct SignalRetainedExecutionBasis {
    pub(crate) storage: SignalExecutionBasis,
    _reservation: SignalConditionalRetentionReservation,
}

impl SignalRetainedExecutionBasis {
    #[cfg(test)]
    pub(crate) fn retained_node_observation(
        &self,
        node: crate::data::handle::NodeId,
        aspect: crate::data::aspect::Aspect,
    ) -> Option<(crate::data::node::NodeState, u64, bool)> {
        self.storage.retained_node_observation(node, aspect)
    }

    #[cfg(test)]
    pub(crate) fn retention_usage(&self) -> (usize, u64) {
        self._reservation.ledger().usage()
    }

    pub(crate) fn new_evaluation_partition(
        &self,
        work: &mut RetainedStoragePreparation,
    ) -> Result<
        crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,
        crate::data::error::SignalError,
    > {
        self.storage.try_new_evaluation_partition(work)
    }

    pub(crate) fn new_evaluation_partition_with_reserved_slot(
        &self,
        work: &mut RetainedStoragePreparation,
        admission: &mut SignalConditionalRetentionReservation,
    ) -> Result<
        crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,
        crate::data::error::SignalError,
    > {
        self.storage
            .try_new_evaluation_partition_with_reserved_slot(work, admission)
    }

    pub(crate) fn fork_evaluation_partition_from(
        &self,
        predecessor: &mut crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,
        work: &mut RetainedStoragePreparation,
        admission: &mut SignalConditionalRetentionReservation,
    ) -> Result<
        crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,
        crate::data::error::SignalError,
    > {
        self.storage
            .try_fork_evaluation_partition_from(predecessor, work, admission)
    }

    pub(crate) fn reserve_evaluation_admission(
        &self,
        charge: Charge,
    ) -> Result<SignalConditionalRetentionReservation, SignalConditionalRetentionDenial> {
        let prepaid_slot_handle = Charge::capacity::<SignalConditionalRetentionReservation>(1)
            .map_err(|_| SignalConditionalRetentionDenial::CapacityExhausted)?;
        self._reservation.ledger().reserve(
            1,
            charge
                .checked_add(prepaid_slot_handle)
                .map_err(|_| SignalConditionalRetentionDenial::CapacityExhausted)?,
        )
    }

    pub(crate) fn capture(
        graph: &mut SignalGraph,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        work: &mut RetainedStoragePreparation,
    ) -> Result<Self, SignalConditionalBasisCaptureDenial> {
        let prepared =
            SignalExecutionBasis::prepare_capture(graph, work).map_err(map_preparation)?;
        let charges = prepared.charges();
        // The parent is temporary source-conversion custody. Its handle is
        // charged by reserve; prepay the returned basis handle before splitting.
        let total = charges
            .retained
            .checked_add(charges.source_growth)
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<SignalConditionalRetentionReservation>(
                    1,
                )?)
            })
            .map_err(|_| SignalConditionalBasisCaptureDenial::CapacityExhausted)?;
        let mut source = ledger.reserve(0, total).map_err(map_retention)?;
        let reservation = source.split(0, charges.retained).map_err(map_retention)?;
        let storage = prepared.capture(&mut source);
        Ok(Self {
            storage,
            _reservation: reservation,
        })
    }
}

fn map_preparation(
    denial: SignalExecutionBasisChargeDenial,
) -> SignalConditionalBasisCaptureDenial {
    match denial {
        SignalExecutionBasisChargeDenial::Storage(
            RetainedStoragePreparationDenial::WorkExhausted { maximum_visits },
        ) => SignalConditionalBasisCaptureDenial::WorkExhausted { maximum_visits },
        SignalExecutionBasisChargeDenial::Storage(
            RetainedStoragePreparationDenial::ChargeOverflow,
        ) => SignalConditionalBasisCaptureDenial::CapacityExhausted,
        _ => SignalConditionalBasisCaptureDenial::Unavailable,
    }
}

fn map_retention(denial: SignalConditionalRetentionDenial) -> SignalConditionalBasisCaptureDenial {
    match denial {
        SignalConditionalRetentionDenial::CapacityExhausted => {
            SignalConditionalBasisCaptureDenial::CapacityExhausted
        }
        SignalConditionalRetentionDenial::Closed => {
            SignalConditionalBasisCaptureDenial::OwnerUnavailable
        }
        SignalConditionalRetentionDenial::Poisoned
        | SignalConditionalRetentionDenial::InvalidTransfer => {
            SignalConditionalBasisCaptureDenial::Unavailable
        }
    }
}
