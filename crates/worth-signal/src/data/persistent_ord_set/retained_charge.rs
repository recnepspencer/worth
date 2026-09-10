use super::PersistentOrdSet;
use crate::data::persistent_ord_map::{RetainedMapMutationDenial, RetainedMapMutationOutcome};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl<T: Clone + Ord> PersistentOrdSet<T> {
    pub(crate) fn prepared_retained_charge(&self) -> Result<Charge, RetainedMapMutationDenial> {
        self.values.prepared_retained_charge()
    }
}

impl<T: Clone + Ord + RetainedStorageMeasurement> PersistentOrdSet<T> {
    pub(crate) fn prepare_retained_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        self.values.prepare_retained_charge(work)
    }

    pub(crate) fn insert_with_retained_charge(
        &mut self,
        value: T,
        work: &mut Preparation,
    ) -> Result<RetainedMapMutationOutcome<bool>, RetainedMapMutationDenial> {
        self.values
            .insert_with_retained_charge(value, (), work)
            .map(|outcome| map_membership_change(outcome, Option::is_none))
    }

    pub(crate) fn remove_with_retained_charge(
        &mut self,
        value: &T,
        work: &mut Preparation,
    ) -> Result<RetainedMapMutationOutcome<bool>, RetainedMapMutationDenial> {
        self.values
            .remove_with_retained_charge(value, work)
            .map(|outcome| map_membership_change(outcome, Option::is_some))
    }
}

fn map_membership_change(
    outcome: RetainedMapMutationOutcome<Option<()>>,
    changed: fn(&Option<()>) -> bool,
) -> RetainedMapMutationOutcome<bool> {
    match outcome {
        RetainedMapMutationOutcome::Accounted { output, charge } => {
            RetainedMapMutationOutcome::Accounted {
                output: changed(&output),
                charge,
            }
        }
        RetainedMapMutationOutcome::Unaccounted { output, denial } => {
            RetainedMapMutationOutcome::Unaccounted {
                output: changed(&output),
                denial,
            }
        }
    }
}

impl<T: Clone + Ord + RetainedStorageMeasurement> RetainedStorageMeasurement
    for PersistentOrdSet<T>
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.values.retained_heap_charge(work)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl<T: Clone + Ord + RetainedStorageMeasurement> RetainedStorageForkPreparation
    for PersistentOrdSet<T>
{
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        self.values.prepare_fork_charge(work)
    }
}
