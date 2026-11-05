//! The runtime's consumer methods over `domain_computation::primary_graph::conditional_operation`; each only calls its subsystem.
use crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryConditionalTruthBasis;
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial;
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeInstallationDenialKind;
use crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeInspection;
use crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeLifecycleProbe;
use crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeReinstallationReceipt;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::Arc;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledPackageIndex;
use worth_query_installation::facade::WorthQueryInstalledPackageIndexRelation;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn inspect_conditional_runtime(&self) -> WorthQueryConditionalRuntimeInspection {
        <Self as crate::domain_computation::primary_graph::conditional_operation::InspectionOps<
            Schema,
        >>::inspect_conditional_runtime(self)
    }

    pub fn conditional_runtime_lifecycle_probe(
        &self,
    ) -> crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeLifecycleProbe{
        <Self as crate::domain_computation::primary_graph::conditional_operation::InspectionOps<
            Schema,
        >>::conditional_runtime_lifecycle_probe(self)
    }

    pub fn close_conditional_runtime(
        &mut self,
    ) -> Result<
        WorthQueryConditionalRuntimeInspection,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        <Self as crate::domain_computation::primary_graph::conditional_operation::InspectionOps<
            Schema,
        >>::close_conditional_runtime(self)
    }

    pub fn reinstall_conditional_runtime(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        WorthQueryConditionalRuntimeReinstallationReceipt,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        <Self as crate::domain_computation::primary_graph::conditional_operation::ReinstallationOps<Schema>>::reinstall_conditional_runtime(self, execution, product_branch)
    }

    pub fn reinstall_conditional_runtime_for_installation(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: Arc<WorthQueryInstalledPackageIndex>,
        product_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        WorthQueryConditionalRuntimeReinstallationReceipt,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        <Self as crate::domain_computation::primary_graph::conditional_operation::ReinstallationOps<Schema>>::reinstall_conditional_runtime_for_installation(self, execution, candidate, product_branch)
    }
}
