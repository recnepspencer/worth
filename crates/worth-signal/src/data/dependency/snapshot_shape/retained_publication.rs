use super::{insertion::PreparedShapeInsertion, DependencySnapshotShapeStore, SnapshotShapeHandle};
use crate::data::error::SignalError;
use crate::data::persistent_ord_map::{RetainedMapMutationDenial, RetainedMapMutationOutcome};
use crate::data::persistent_vector::{
    RetainedVectorMutationDenial, RetainedVectorMutationOutcome, RetainedVectorStagingDenial,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};

impl PreparedShapeInsertion {
    pub(in crate::data::dependency) fn retained_staging_charge(
        &self,
        store: &DependencySnapshotShapeStore,
        work: &mut Work,
    ) -> Result<Charge, SignalError> {
        let Some((shape, _)) = &self.append else {
            return Ok(Charge::ZERO);
        };
        store
            .shapes
            .prepare_push_staging_charge(shape, work)
            .map_err(map_vector_staging)?
            .checked_add(
                store
                    .interner
                    .prepare_insert_staging_charge(shape, &self.handle, work)
                    .map_err(map_map_mutation)?,
            )
            .map_err(map_accounting)
    }

    pub(in crate::data::dependency) fn publish_retained(
        self,
        store: &mut DependencySnapshotShapeStore,
        work: &mut Work,
    ) -> Result<SnapshotShapeHandle, SignalError> {
        if let Some((shape, expected_len)) = self.append {
            assert_eq!(
                store.shapes.len(),
                expected_len,
                "shape insertion must remain inside exclusive preparation/publication"
            );
            accounted_vector(store.shapes.push_with_retained_charge(shape.clone(), work))?;
            accounted_map(
                store
                    .interner
                    .insert_with_retained_charge(shape, self.handle, work),
            )?;
        }
        Ok(self.handle)
    }
}

impl DependencySnapshotShapeStore {
    pub(crate) fn prepared_retained_charge(&self) -> Result<Charge, SignalError> {
        self.shapes
            .prepared_retained_charge()
            .map_err(map_vector_mutation)?
            .checked_add(
                self.interner
                    .prepared_retained_charge()
                    .map_err(map_map_mutation)?,
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
