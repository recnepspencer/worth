use super::{ForkConsumerMembershipChange, ForkConsumerMemberships};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for ForkConsumerMemberships {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(memberships) = self;
        memberships.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ForkConsumerMembershipChange {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Removed => Ok(Charge::ZERO),
            Self::Replaced(memberships) => memberships.retained_heap_charge(work),
        }
    }
}
