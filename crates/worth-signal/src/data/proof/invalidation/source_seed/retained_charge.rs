use super::DirectInvalidationBasis;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for DirectInvalidationBasis {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::InitialCompute { generation: _ } => Ok(Charge::ZERO),
            Self::SourceRecompute {
                generation: _,
                dirty_aspects: _,
                scoped_aspects,
            } => scoped_aspects.retained_heap_charge(work),
        }
    }
}
