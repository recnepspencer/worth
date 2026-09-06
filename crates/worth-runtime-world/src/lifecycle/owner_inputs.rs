use worth_relational::facade::branch::RelationalOwnerServicePorts;
use worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort;
use worth_signal::facade::branch::SignalOwnerServicePorts;

use crate::budget::RuntimeWorldBudgets;

use super::RuntimeWorldClock;

/// Concrete composition inputs issued by the two component owners and Bridge.
///
/// The later managed owner receives exactly these already-issued bundles; it
/// never accepts raw runtimes or recreates an owner seal.
pub struct RuntimeWorldOwnerInputs<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    relational: RelationalOwnerServicePorts,
    signal: SignalOwnerServicePorts<D, I, E, Ctx, T>,
    bridge: RuntimeWorldCorrespondencePort,
    budgets: RuntimeWorldBudgets,
    clock: RuntimeWorldClock,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerInputs<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn new(
        relational: RelationalOwnerServicePorts,
        signal: SignalOwnerServicePorts<D, I, E, Ctx, T>,
        bridge: RuntimeWorldCorrespondencePort,
        budgets: RuntimeWorldBudgets,
        clock: RuntimeWorldClock,
    ) -> Self {
        Self {
            relational,
            signal,
            bridge,
            budgets,
            clock,
        }
    }

    #[cfg(test)]
    pub fn relational(&self) -> &RelationalOwnerServicePorts {
        &self.relational
    }

    #[cfg(test)]
    pub fn signal(&self) -> &SignalOwnerServicePorts<D, I, E, Ctx, T> {
        &self.signal
    }

    #[cfg(test)]
    pub fn bridge(&self) -> &RuntimeWorldCorrespondencePort {
        &self.bridge
    }

    #[cfg(test)]
    pub fn budgets(&self) -> &RuntimeWorldBudgets {
        &self.budgets
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RelationalOwnerServicePorts,
        SignalOwnerServicePorts<D, I, E, Ctx, T>,
        RuntimeWorldCorrespondencePort,
        RuntimeWorldBudgets,
        RuntimeWorldClock,
    ) {
        (
            self.relational,
            self.signal,
            self.bridge,
            self.budgets,
            self.clock,
        )
    }
}
