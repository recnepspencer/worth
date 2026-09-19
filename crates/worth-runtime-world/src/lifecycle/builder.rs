use worth_relational::facade::branch::RelationalOwnerServicePorts;
use worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort;
use worth_signal::facade::branch::{
    SignalConditionalDefinitionPublicationPort, SignalOwnerServicePorts,
};

use super::public_owner::RuntimeWorldOwner;
use super::{RuntimeWorldClock, RuntimeWorldOwnerInputs};
use crate::budget::RuntimeWorldBudgets;
use crate::identity::RuntimeWorldIdentityExhaustion;

pub struct MissingRuntimeWorldInput;

pub struct RuntimeWorldOwnerBuilder<B, R, S, P, U, C> {
    bridge: B,
    relational: R,
    signal: S,
    signal_definition_publication: P,
    budgets: U,
    clock: C,
}

impl
    RuntimeWorldOwnerBuilder<
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
    >
{
    pub(super) fn new() -> Self {
        Self {
            bridge: MissingRuntimeWorldInput,
            relational: MissingRuntimeWorldInput,
            signal: MissingRuntimeWorldInput,
            signal_definition_publication: MissingRuntimeWorldInput,
            budgets: MissingRuntimeWorldInput,
            clock: MissingRuntimeWorldInput,
        }
    }
}

impl<R, S, P, U, C> RuntimeWorldOwnerBuilder<MissingRuntimeWorldInput, R, S, P, U, C> {
    pub fn with_bridge_correspondence(
        self,
        bridge: RuntimeWorldCorrespondencePort,
    ) -> RuntimeWorldOwnerBuilder<RuntimeWorldCorrespondencePort, R, S, P, U, C> {
        RuntimeWorldOwnerBuilder {
            bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock: self.clock,
        }
    }
}

impl<B, S, P, U, C> RuntimeWorldOwnerBuilder<B, MissingRuntimeWorldInput, S, P, U, C> {
    pub fn with_relational_services(
        self,
        relational: RelationalOwnerServicePorts,
    ) -> RuntimeWorldOwnerBuilder<B, RelationalOwnerServicePorts, S, P, U, C> {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock: self.clock,
        }
    }
}

impl<B, R, P, U, C> RuntimeWorldOwnerBuilder<B, R, MissingRuntimeWorldInput, P, U, C> {
    pub fn with_signal_services<D, I, E, Ctx, T>(
        self,
        signal: SignalOwnerServicePorts<D, I, E, Ctx, T>,
    ) -> RuntimeWorldOwnerBuilder<B, R, SignalOwnerServicePorts<D, I, E, Ctx, T>, P, U, C>
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock: self.clock,
        }
    }
}

impl<B, R, D, I, E, Ctx, T, U, C>
    RuntimeWorldOwnerBuilder<
        B,
        R,
        SignalOwnerServicePorts<D, I, E, Ctx, T>,
        MissingRuntimeWorldInput,
        U,
        C,
    >
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn with_signal_definition_publication(
        self,
        publication: SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
    ) -> RuntimeWorldOwnerBuilder<
        B,
        R,
        SignalOwnerServicePorts<D, I, E, Ctx, T>,
        SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
        U,
        C,
    > {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: publication,
            budgets: self.budgets,
            clock: self.clock,
        }
    }
}

impl<B, R, S, P, C> RuntimeWorldOwnerBuilder<B, R, S, P, MissingRuntimeWorldInput, C> {
    pub fn with_budgets(
        self,
        budgets: RuntimeWorldBudgets,
    ) -> RuntimeWorldOwnerBuilder<B, R, S, P, RuntimeWorldBudgets, C> {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets,
            clock: self.clock,
        }
    }
}

impl<B, R, S, P, U> RuntimeWorldOwnerBuilder<B, R, S, P, U, MissingRuntimeWorldInput> {
    pub fn with_clock(
        self,
        clock: RuntimeWorldClock,
    ) -> RuntimeWorldOwnerBuilder<B, R, S, P, U, RuntimeWorldClock> {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock,
        }
    }
}

impl<D, I, E, Ctx, T>
    RuntimeWorldOwnerBuilder<
        RuntimeWorldCorrespondencePort,
        RelationalOwnerServicePorts,
        SignalOwnerServicePorts<D, I, E, Ctx, T>,
        SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
        RuntimeWorldBudgets,
        RuntimeWorldClock,
    >
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn build(
        self,
    ) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, RuntimeWorldIdentityExhaustion> {
        RuntimeWorldOwner::from_inputs(RuntimeWorldOwnerInputs::new(
            self.relational,
            self.signal,
            self.signal_definition_publication,
            self.bridge,
            self.budgets,
            self.clock,
        ))
    }
}
