//! Read-only owner observation of an installed equality index's generation
//! residency on the live root product branch without triggering cold I/O or
//! reconstruction.

use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::indexes::DerivedIndexGenerationResidency;
use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};

use super::WorthQueryPrimaryGraphApplicationRuntimeInner;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
    /// Observe the current root branch's generation residency for a declared
    /// field's installed equality index without mutating state or triggering
    /// cold reconstruction.
    ///
    /// Returns `Ok(None)` when the field has no installed equality index in
    /// this program's schema layout, or `Ok(Some(residency))` reporting the
    /// exact Native owner residency (`Published`, `DeferredImage`, or `Absent`).
    pub fn observe_equality_index_residency<F: RecordSelection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: F,
    ) -> Result<
        Option<DerivedIndexGenerationResidency>,
        RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>,
    > {
        let head = self
            .runtime
            .product_runtime
            .owner_ref()
            .observation_port()
            .observe_root_product_branch()?;
        let graph = self
            .runtime
            .retain_primary_graph_integration_handle()
            .expect("installed primary graph");
        Ok(graph.with_query_runtime(execution, |runtime, layout| {
            let index =
                layout.equality_selector_index(field.entity(), field.aspect(), field.field())?;
            Some(runtime.index_access().generation_residency_for_observation(
                index,
                &head.basis().relational_basis().observation(),
            ))
        }))
    }
}
