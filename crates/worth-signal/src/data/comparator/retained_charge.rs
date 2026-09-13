use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::VersionComparatorPolicy;

impl RetainedStorageMeasurement for VersionComparatorPolicy {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Custom { key } => key.retained_heap_charge(work),
            Self::Exact
            | Self::Tolerance { epsilon: _ }
            | Self::OutputIdentity
            | Self::Installed { identity: _ } => Ok(Charge::ZERO),
        }
    }
}
