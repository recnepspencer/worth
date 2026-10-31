//! Authored owner configuration before Home binds its actual Native source.
use worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder;
use worth_runtime_world::facade::RuntimeWorldHomeBuilder;

/// Query supplies authored configuration, never independently completed owners.
pub struct ApplicationHomeWorldDefinition {
    pub(super) bridge: UnattachedRelationalBridgeBuilder,
    pub(super) graph_role: String,
    pub(super) budgets: worth_runtime_world::facade::RuntimeWorldBudgets,
    pub(super) clock: worth_runtime_world::facade::RuntimeWorldClock,
    pub(super) execution_policy: worth_foundational::facade::ExecutionRequestPolicy,
    pub(super) execution_authority: Option<std::sync::Arc<worth_execution::ExecutionAuthority>>,
    pub(super) conditional_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
}
impl ApplicationHomeWorldDefinition {
    pub fn new(
        bridge: UnattachedRelationalBridgeBuilder,
        graph_role: impl Into<String>,
        budgets: worth_runtime_world::facade::RuntimeWorldBudgets,
        clock: worth_runtime_world::facade::RuntimeWorldClock,
        execution_policy: worth_foundational::facade::ExecutionRequestPolicy,
        execution_authority: Option<std::sync::Arc<worth_execution::ExecutionAuthority>>,
        conditional_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) -> Self {
        Self {
            bridge,
            graph_role: graph_role.into(),
            budgets,
            clock,
            execution_policy,
            execution_authority,
            conditional_budget,
        }
    }
}

pub struct ApplicationWorldDefinition<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) world: RuntimeWorldHomeBuilder<D, I, E, Ctx, T>,
    pub(super) bridge: UnattachedRelationalBridgeBuilder,
    pub(super) graph_role: String,
}
impl<D, I, E, Ctx, T> ApplicationWorldDefinition<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn new(
        world: RuntimeWorldHomeBuilder<D, I, E, Ctx, T>,
        bridge: UnattachedRelationalBridgeBuilder,
        graph_role: impl Into<String>,
    ) -> Self {
        Self {
            world,
            bridge,
            graph_role: graph_role.into(),
        }
    }
}
