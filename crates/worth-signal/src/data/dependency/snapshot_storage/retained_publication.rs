use super::{
    insertion::InsertionAction, DependencySnapshotShapeStore, DependencySnapshotStore,
    PreparedSnapshotInsertion, SnapshotShapeHandle,
};
use crate::data::error::SignalError;
use crate::data::persistent_ord_map::{RetainedMapMutationDenial, RetainedMapMutationOutcome};
use crate::data::persistent_vector::{
    RetainedVectorMutationDenial, RetainedVectorMutationOutcome, RetainedVectorStagingDenial,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};

impl PreparedSnapshotInsertion {
    pub(crate) fn changes_storage(&self) -> bool {
        matches!(self.action, InsertionAction::Append { .. })
    }

    pub(crate) fn retained_staging_charge(
        &self,
        store: &DependencySnapshotStore,
        shapes: &DependencySnapshotShapeStore,
        work: &mut Work,
    ) -> Result<Charge, SignalError> {
        let InsertionAction::Append {
            snapshot, shape, ..
        } = &self.action
        else {
            return Ok(Charge::ZERO);
        };
        let mut charge = store
            .snapshots
            .prepare_push_staging_charge(snapshot, work)
            .map_err(map_vector_staging)?;
        charge = charge
            .checked_add(
                store
                    .shape_handles
                    .prepare_push_staging_charge(&shape.handle(), work)
                    .map_err(map_vector_staging)?,
            )
            .map_err(map_accounting)?;
        charge = charge
            .checked_add(
                store
                    .interner
                    .prepare_insert_staging_charge(snapshot, &self.id, work)
                    .map_err(map_map_mutation)?,
            )
            .map_err(map_accounting)?;
        charge
            .checked_add(shape.retained_staging_charge(shapes, work)?)
            .map_err(map_accounting)
    }

    pub(crate) fn publish_retained(
        self,
        store: &mut DependencySnapshotStore,
        shapes: &mut DependencySnapshotShapeStore,
        work: &mut Work,
    ) -> Result<(super::DependencySnapshotId, SnapshotShapeHandle), SignalError> {
        let handle = match self.action {
            InsertionAction::Existing(handle) => handle,
            InsertionAction::Append {
                snapshot,
                shape,
                expected_len,
            } => {
                assert_eq!(
                    store.snapshots.len(),
                    expected_len,
                    "snapshot insertion must remain inside exclusive preparation/publication"
                );
                let handle = shape.publish_retained(shapes, work)?;
                accounted_vector(
                    store
                        .snapshots
                        .push_with_retained_charge(snapshot.clone(), work),
                )?;
                accounted_vector(store.shape_handles.push_with_retained_charge(handle, work))?;
                accounted_map(
                    store
                        .interner
                        .insert_with_retained_charge(snapshot, self.id, work),
                )?;
                handle
            }
        };
        Ok((self.id, handle))
    }
}

impl DependencySnapshotStore {
    pub(crate) fn prepared_retained_charge(&self) -> Result<Charge, SignalError> {
        let mut charge = self
            .snapshots
            .prepared_retained_charge()
            .map_err(map_vector_mutation)?;
        charge = charge
            .checked_add(
                self.interner
                    .prepared_retained_charge()
                    .map_err(map_map_mutation)?,
            )
            .map_err(map_accounting)?;
        charge
            .checked_add(
                self.shape_handles
                    .prepared_retained_charge()
                    .map_err(map_vector_mutation)?,
            )
            .map_err(map_accounting)
    }
}

fn accounted_vector<R>(
    outcome: Result<RetainedVectorMutationOutcome<R>, RetainedVectorMutationDenial>,
) -> Result<R, SignalError> {
    match outcome.map_err(map_vector_mutation)? {
        RetainedVectorMutationOutcome::Accounted { output, .. } => Ok(output),
        RetainedVectorMutationOutcome::Unaccounted { denial, .. } => Err(map_accounting(denial)),
    }
}

fn accounted_map<R>(
    outcome: Result<RetainedMapMutationOutcome<R>, RetainedMapMutationDenial>,
) -> Result<R, SignalError> {
    match outcome.map_err(map_map_mutation)? {
        RetainedMapMutationOutcome::Accounted { output, .. } => Ok(output),
        RetainedMapMutationOutcome::Unaccounted { denial, .. } => Err(map_accounting(denial)),
    }
}

fn map_vector_staging(denial: RetainedVectorStagingDenial) -> SignalError {
    match denial {
        RetainedVectorStagingDenial::Mutation(denial) => map_vector_mutation(denial),
        RetainedVectorStagingDenial::ForkPreparationRequired => {
            SignalError::SnapshotIndexUnavailable
        }
    }
}

fn map_vector_mutation(denial: RetainedVectorMutationDenial) -> SignalError {
    match denial {
        RetainedVectorMutationDenial::Accounting(denial) => map_accounting(denial),
        RetainedVectorMutationDenial::PreparationRequired
        | RetainedVectorMutationDenial::MissingElement { .. } => {
            SignalError::SnapshotIndexUnavailable
        }
    }
}

fn map_map_mutation(denial: RetainedMapMutationDenial) -> SignalError {
    match denial {
        RetainedMapMutationDenial::Accounting(denial) => map_accounting(denial),
        RetainedMapMutationDenial::PreparationRequired | RetainedMapMutationDenial::MissingKey => {
            SignalError::SnapshotIndexUnavailable
        }
    }
}

fn map_accounting(denial: RetainedStoragePreparationDenial) -> SignalError {
    match denial {
        RetainedStoragePreparationDenial::WorkExhausted { maximum_visits } => {
            SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }
        }
        _ => SignalError::EvaluationStorageCapacityExhausted,
    }
}
