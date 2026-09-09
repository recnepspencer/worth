use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{NodeContract, NodeProjectionContract, NodeSemanticContract};

impl RetainedStorageMeasurement for NodeContract {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            semantics,
            projection,
            reuse,
            execution: _,
            authority: _,
        } = self;
        Ok(Charge::ZERO
            .checked_add(semantics.retained_heap_charge(work)?)?
            .checked_add(projection.retained_heap_charge(work)?)?
            .checked_add(reuse.retained_heap_charge(work)?)?)
    }
}

impl RetainedStorageMeasurement for NodeSemanticContract {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            partition_scope,
            reads: _,
            produces: _,
            required_context: _,
        } = self;
        Ok(Charge::ZERO.checked_add(partition_scope.retained_heap_charge(work)?)?)
    }
}

impl RetainedStorageMeasurement for NodeProjectionContract {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            consumes_partitions,
            consumes: _,
        } = self;
        Ok(Charge::ZERO.checked_add(consumes_partitions.retained_heap_charge(work)?)?)
    }
}

impl RetainedStorageMeasurement for super::ContextRequirement {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::None | Self::DomainContext | Self::RelationalSnapshot => Ok(Charge::ZERO),
        }
    }
}
