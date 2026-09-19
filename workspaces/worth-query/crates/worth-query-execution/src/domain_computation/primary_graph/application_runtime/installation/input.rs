use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};

pub(in crate::domain_computation::primary_graph) struct ApplicationRuntimePublication<Schema> {
    pub(in crate::domain_computation::primary_graph) bootstrap:
        super::super::WorthQueryPrimaryGraphBootstrap<Schema>,
    pub(in crate::domain_computation::primary_graph) runtime: WorthQueryExecutionRuntime,
    pub(in crate::domain_computation::primary_graph) authority:
        WorthQueryExecutionInstallationAuthority,
    pub(in crate::domain_computation::primary_graph) installed_schema:
        WorthQueryInstalledApplicationSchema<Schema>,
    pub(in crate::domain_computation::primary_graph) authorization_clock: WorthQueryRuntimeClock,
    pub(in crate::domain_computation::primary_graph) fault_port:
        std::sync::Arc<dyn crate::domain_computation::primary_graph::provider::fault_port::WorthQueryPrimaryGraphFaultPort>,
    pub(in crate::domain_computation::primary_graph) conditional_evaluation_budget:
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
}
