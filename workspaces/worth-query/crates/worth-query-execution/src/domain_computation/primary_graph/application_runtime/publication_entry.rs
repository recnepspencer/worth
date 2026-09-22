//! The publication entry points one prepared primary-graph bootstrap offers.
//!
//! Every path here consumes the bootstrap together with the execution root and
//! its installation authority, so a published application runtime can only ever
//! come from a graph that was prepared for it. The paths differ only in which
//! trusted-time mechanism and which fault port the resulting runtime is fixed
//! to, and in whether conditional bindings must still be accumulated first.

use std::sync::Arc;

use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};
use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource;

use super::super::conditional_operation::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};
use super::super::provider::fault_port::{production_fault_port, WorthQueryPrimaryGraphFaultPort};
use super::super::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use super::installation;

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn publish_application_runtime(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: SignalConditionalEvaluationBudget,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        installation::require_no_conditional_bindings(&runtime, &installed_schema)?;
        installation::publish_application_runtime_with_clock(
            installation::ApplicationRuntimePublication {
                bootstrap: self,
                runtime,
                authority,
                installed_schema,
                authorization_clock: WorthQueryRuntimeClock::system(),
                fault_port: production_fault_port(),
                conditional_evaluation_budget,
            },
        )
    }

    /// Begins the sole primary-graph conditional publication progression.
    ///
    /// Complete provider, clock, and reconstruction bindings are accumulated
    /// here before the application runtime can become visible.
    pub fn conditional_application_runtime_installation(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: SignalConditionalEvaluationBudget,
    ) -> Result<
        WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        WorthQueryConditionalApplicationRuntimeInstallation::new(
            installation::ApplicationRuntimePublication {
                bootstrap: self,
                runtime,
                authority,
                installed_schema,
                authorization_clock: WorthQueryRuntimeClock::system(),
                fault_port: production_fault_port(),
                conditional_evaluation_budget,
            },
        )
    }

    /// Begins conditional publication with one host-installed trusted-time
    /// mechanism fixed for the lifetime of the resulting runtime.
    pub fn conditional_application_runtime_installation_with_authorization_time_source(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: SignalConditionalEvaluationBudget,
        source: impl WorthQueryRuntimeTimeSource,
    ) -> Result<
        WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        WorthQueryConditionalApplicationRuntimeInstallation::new(
            installation::ApplicationRuntimePublication {
                bootstrap: self,
                runtime,
                authority,
                installed_schema,
                authorization_clock: WorthQueryRuntimeClock::from_source(source),
                fault_port: production_fault_port(),
                conditional_evaluation_budget,
            },
        )
    }

    /// Publishes one application runtime with a host-installed trusted-time
    /// mechanism.
    ///
    /// The source is fixed for the lifetime of the returned runtime. It grants
    /// no Query authority and is never exposed to operation callers.
    pub fn publish_application_runtime_with_authorization_time_source(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: SignalConditionalEvaluationBudget,
        source: impl WorthQueryRuntimeTimeSource,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        self.publish_application_runtime_with_ports(
            runtime,
            authority,
            installed_schema,
            conditional_evaluation_budget,
            source,
            production_fault_port(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn publish_application_runtime_with_ports(
        self,
        runtime: WorthQueryExecutionRuntime,
        authority: WorthQueryExecutionInstallationAuthority,
        installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
        conditional_evaluation_budget: SignalConditionalEvaluationBudget,
        source: impl WorthQueryRuntimeTimeSource,
        fault_port: Arc<dyn WorthQueryPrimaryGraphFaultPort>,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        installation::publish_application_runtime_with_clock(
            installation::ApplicationRuntimePublication {
                bootstrap: self,
                runtime,
                authority,
                installed_schema,
                authorization_clock: WorthQueryRuntimeClock::from_source(source),
                fault_port,
                conditional_evaluation_budget,
            },
        )
    }
}
