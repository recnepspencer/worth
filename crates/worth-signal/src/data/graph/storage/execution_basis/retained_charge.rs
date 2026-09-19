use super::SignalExecutionBasis;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparationDenial,
};
#[cfg(test)]
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
};
use crate::diagnostics::state::BranchCarrierChargeDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalExecutionBasisChargeDenial {
    TopologyIndexes(crate::data::graph::signal_graph::RetainedTopologyIndexDenial),
    Storage(RetainedStoragePreparationDenial),
    Diagnostics(BranchCarrierChargeDenial),
}

impl From<crate::data::graph::signal_graph::RetainedTopologyIndexDenial>
    for SignalExecutionBasisChargeDenial
{
    fn from(value: crate::data::graph::signal_graph::RetainedTopologyIndexDenial) -> Self {
        Self::TopologyIndexes(value)
    }
}

impl From<RetainedStoragePreparationDenial> for SignalExecutionBasisChargeDenial {
    fn from(value: RetainedStoragePreparationDenial) -> Self {
        Self::Storage(value)
    }
}

impl From<BranchCarrierChargeDenial> for SignalExecutionBasisChargeDenial {
    fn from(value: BranchCarrierChargeDenial) -> Self {
        Self::Diagnostics(value)
    }
}

impl SignalExecutionBasis {
    #[cfg(test)]
    pub(super) fn measure_seed_charge(
        &self,
        work: &mut Preparation,
    ) -> Result<Charge, SignalExecutionBasisChargeDenial> {
        work.visit()?;
        let Self {
            definitions,
            evaluation,
            retained_charge: _,
        } = self;
        Ok(Charge::capacity::<Self>(1)?
            .checked_add(definitions.retained_heap_charge(work)?)?
            .checked_add(evaluation.measure_seed_heap_charge(work)?)?)
    }

    /// Immutable prepared charge, including this value and reachable storage.
    /// An owning allocation must additionally charge its own header. This
    /// does not describe a mutable slot after it has performed work.
    pub(crate) fn retained_storage_charge(&self) -> Charge {
        self.retained_charge
    }
}
