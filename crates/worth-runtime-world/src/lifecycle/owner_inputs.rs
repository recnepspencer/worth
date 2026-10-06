use std::sync::Arc;

use worth_execution::ExecutionAuthority;
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
    pub(crate) relational: RelationalOwnerServicePorts,
    pub(crate) signal: SignalOwnerServicePorts<D, I, E, Ctx, T>,
    pub(crate) signal_definition_publication:
        worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
    pub(crate) bridge: RuntimeWorldCorrespondencePort,
    pub(crate) budgets: RuntimeWorldBudgets,
    pub(crate) clock: RuntimeWorldClock,
    pub(crate) execution_authority: Option<Arc<ExecutionAuthority>>,
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
        signal_definition_publication: worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<
            D, I, E, Ctx, T,
        >,
        bridge: RuntimeWorldCorrespondencePort,
        budgets: RuntimeWorldBudgets,
        clock: RuntimeWorldClock,
    ) -> Self {
        Self {
            relational,
            signal,
            signal_definition_publication,
            bridge,
            budgets,
            clock,
            execution_authority: None,
        }
    }

    /// The host shares its sole process execution authority with this World.
    pub fn with_execution_authority(mut self, authority: Arc<ExecutionAuthority>) -> Self {
        self.execution_authority = Some(authority);
        self
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
}
