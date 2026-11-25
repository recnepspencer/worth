//! Forwarding for read-only equality index generation residency observation.

use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::indexes::DerivedIndexGenerationResidency;
use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
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
        self.inner
            .observe_equality_index_residency(execution, field)
    }
}
