use std::sync::Arc;

use worth_execution::ExecutionAuthority;
use worth_foundational::ExecutionRequestPolicy;
use worth_relational::facade::branch::RelationalOwnerServicePorts;
use worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort;
use worth_signal::facade::branch::{
    SignalConditionalDefinitionPublicationPort, SignalOwnerServicePorts,
};

use super::execution::{InstalledExecution, RuntimeWorldBuildDenial};
use super::public_owner::RuntimeWorldOwner;
use super::{RuntimeWorldClock, RuntimeWorldOwnerInputs};
use crate::budget::RuntimeWorldBudgets;

pub struct MissingRuntimeWorldInput;

pub struct RuntimeWorldOwnerBuilder<B, R, S, P, U, C, X = MissingRuntimeWorldInput> {
    bridge: B,
    relational: R,
    signal: S,
    signal_definition_publication: P,
    budgets: U,
    clock: C,
    execution_authority: Option<Arc<ExecutionAuthority>>,
    execution_policy: X,
}

impl
    RuntimeWorldOwnerBuilder<
        MissingRuntimeWorldInput,
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
            execution_authority: None,
            execution_policy: MissingRuntimeWorldInput,
        }
    }
}

impl<R, S, P, U, C, X> RuntimeWorldOwnerBuilder<MissingRuntimeWorldInput, R, S, P, U, C, X> {
    pub fn with_bridge_correspondence(
        self,
        bridge: RuntimeWorldCorrespondencePort,
    ) -> RuntimeWorldOwnerBuilder<RuntimeWorldCorrespondencePort, R, S, P, U, C, X> {
        RuntimeWorldOwnerBuilder {
            bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock: self.clock,
            execution_authority: self.execution_authority,
            execution_policy: self.execution_policy,
        }
    }
}

impl<B, S, P, U, C, X> RuntimeWorldOwnerBuilder<B, MissingRuntimeWorldInput, S, P, U, C, X> {
    pub fn with_relational_services(
        self,
        relational: RelationalOwnerServicePorts,
    ) -> RuntimeWorldOwnerBuilder<B, RelationalOwnerServicePorts, S, P, U, C, X> {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock: self.clock,
            execution_authority: self.execution_authority,
            execution_policy: self.execution_policy,
        }
    }
}

impl<B, R, P, U, C, X> RuntimeWorldOwnerBuilder<B, R, MissingRuntimeWorldInput, P, U, C, X> {
    pub fn with_signal_services<D, I, E, Ctx, T>(
        self,
        signal: SignalOwnerServicePorts<D, I, E, Ctx, T>,
    ) -> RuntimeWorldOwnerBuilder<B, R, SignalOwnerServicePorts<D, I, E, Ctx, T>, P, U, C, X>
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
            execution_authority: self.execution_authority,
            execution_policy: self.execution_policy,
        }
    }
}

impl<B, R, D, I, E, Ctx, T, U, C, X>
    RuntimeWorldOwnerBuilder<
        B,
        R,
        SignalOwnerServicePorts<D, I, E, Ctx, T>,
        MissingRuntimeWorldInput,
        U,
        C,
        X,
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
        X,
    > {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: publication,
            budgets: self.budgets,
            clock: self.clock,
            execution_authority: self.execution_authority,
            execution_policy: self.execution_policy,
        }
    }
}

impl<B, R, S, P, C, X> RuntimeWorldOwnerBuilder<B, R, S, P, MissingRuntimeWorldInput, C, X> {
    pub fn with_budgets(
        self,
        budgets: RuntimeWorldBudgets,
    ) -> RuntimeWorldOwnerBuilder<B, R, S, P, RuntimeWorldBudgets, C, X> {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets,
            clock: self.clock,
            execution_authority: self.execution_authority,
            execution_policy: self.execution_policy,
        }
    }
}

impl<B, R, S, P, U, X> RuntimeWorldOwnerBuilder<B, R, S, P, U, MissingRuntimeWorldInput, X> {
    pub fn with_clock(
        self,
        clock: RuntimeWorldClock,
    ) -> RuntimeWorldOwnerBuilder<B, R, S, P, U, RuntimeWorldClock, X> {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock,
            execution_authority: self.execution_authority,
            execution_policy: self.execution_policy,
        }
    }
}

impl<B, R, S, P, U, C, X> RuntimeWorldOwnerBuilder<B, R, S, P, U, C, X> {
    /// Install the host's process authority for the World and its descendants.
    /// Requests lease from it only under a policy installed beside it.
    pub fn with_execution_authority(mut self, authority: Arc<ExecutionAuthority>) -> Self {
        self.execution_authority = Some(authority);
        self
    }

    /// Install the request policy every request runs under: leased from the
    /// authority when one is installed, on the calling thread otherwise.
    pub fn with_execution_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> RuntimeWorldOwnerBuilder<B, R, S, P, U, C, ExecutionRequestPolicy> {
        RuntimeWorldOwnerBuilder {
            bridge: self.bridge,
            relational: self.relational,
            signal: self.signal,
            signal_definition_publication: self.signal_definition_publication,
            budgets: self.budgets,
            clock: self.clock,
            execution_authority: self.execution_authority,
            execution_policy: policy,
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
        ExecutionRequestPolicy,
    >
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn build(self) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, RuntimeWorldBuildDenial> {
        let execution =
            InstalledExecution::try_new(self.execution_authority, self.execution_policy)?;
        let inputs = RuntimeWorldOwnerInputs::new(
            self.relational,
            self.signal,
            self.signal_definition_publication,
            self.bridge,
            self.budgets,
            self.clock,
            execution.request_policy(),
        )
        .with_execution(execution);
        RuntimeWorldOwner::from_inputs(inputs).map_err(RuntimeWorldBuildDenial::IdentityExhaustion)
    }
}
