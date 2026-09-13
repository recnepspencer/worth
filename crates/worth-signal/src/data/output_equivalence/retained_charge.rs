use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::OutputEquivalencePolicy;

impl RetainedStorageMeasurement for OutputEquivalencePolicy {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Custom { key } => key.retained_heap_charge(work),
            Self::ExactAspectVersion
            | Self::AspectVersionTolerance { epsilon: _ }
            | Self::OutputIdentity
            | Self::Installed { identity: _ } => Ok(Charge::ZERO),
        }
    }
}
