//! Staged replacement of a bounded persistent page under a retained ceiling.
use super::{
    PersistentVector, PersistentVectorStorage, RetainedVectorMutationDenial,
    RetainedVectorMutationOutcome,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedVectorStagingDenial {
    ForkPreparationRequired,
    Mutation(RetainedVectorMutationDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedVectorCapacityDenial {
    CapacityExhausted { maximum: Charge, required: Charge },
    Accounting(RetainedStoragePreparationDenial),
}

#[derive(Debug)]
pub(crate) enum RetainedVectorCapacityOutcome<R> {
    Installed {
        output: R,
        charge: Charge,
    },
    Rejected {
        output: R,
        denial: RetainedVectorCapacityDenial,
    },
}

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    /// Exact retained storage preconditions, not value equivalence or runtime
    /// authority. Logical extent and accounting readiness can change without
    /// replacing the immutable backing roots.
    pub(crate) fn matches_retained_edit_preconditions(&self, current: &Self) -> bool {
        self.shares_storage_with(current)
            && self.len() == current.len()
            && matches!((self.prepared_retained_charge(), current.prepared_retained_charge()),
                (Ok(before), Ok(now)) if before == now)
    }

    pub(crate) fn validate_staged_edit(
        &self,
        index: usize,
    ) -> Result<Charge, RetainedVectorStagingDenial> {
        if matches!(self.storage, PersistentVectorStorage::Exclusive(_)) {
            return Err(RetainedVectorStagingDenial::ForkPreparationRequired);
        }
        let charge = self
            .prepared_retained_charge()
            .map_err(RetainedVectorStagingDenial::Mutation)?;
        if self.get(index).is_none() {
            return Err(RetainedVectorStagingDenial::Mutation(
                RetainedVectorMutationDenial::MissingElement { index },
            ));
        }
        Ok(charge)
    }

    /// The caller owns capacity reservation; this enforces the installed
    /// representation ceiling. Draft allocations and returned output require
    /// separate work/resource custody. Atomicity covers vector-owned storage,
    /// not external effects or shared interior mutation performed by `edit`.
    pub(crate) fn edit_with_retained_capacity<R>(
        &mut self,
        index: usize,
        maximum: Charge,
        work: &mut Preparation,
        edit: impl FnOnce(&mut T) -> R,
    ) -> Result<RetainedVectorCapacityOutcome<R>, RetainedVectorStagingDenial> {
        self.validate_staged_edit(index)?;
        // Fork-shared clone retains roots; only the selected page detaches.
        // Unwind or any rejection drops the draft without replacing this root.
        let mut draft = self.clone();
        let outcome = draft
            .edit_with_retained_charge(index, work, edit)
            .map_err(RetainedVectorStagingDenial::Mutation)?;
        Ok(match outcome {
            RetainedVectorMutationOutcome::Accounted { output, charge } if charge <= maximum => {
                *self = draft;
                RetainedVectorCapacityOutcome::Installed { output, charge }
            }
            RetainedVectorMutationOutcome::Accounted { output, charge } => {
                RetainedVectorCapacityOutcome::Rejected {
                    output,
                    denial: RetainedVectorCapacityDenial::CapacityExhausted {
                        maximum,
                        required: charge,
                    },
                }
            }
            RetainedVectorMutationOutcome::Unaccounted { output, denial } => {
                RetainedVectorCapacityOutcome::Rejected {
                    output,
                    denial: RetainedVectorCapacityDenial::Accounting(denial),
                }
            }
        })
    }
}
