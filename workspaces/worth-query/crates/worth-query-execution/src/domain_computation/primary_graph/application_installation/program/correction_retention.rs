//! The actual Home owner expires optional material and settles its interpretation.
use super::program_runtime::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates;
use crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial;
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema, Program> WorthQueryProgramApplicationRuntime<Schema, Program> {
    /// Expire one unpinned original's optional correction payload. Its durable key,
    /// intent, lineage and Native receipt remain. Retry completes the same disposal.
    pub fn expire_optional_correction_material(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        coordinates: &WorthQueryCorrectionSourceCoordinates,
    ) -> Result<(), WorthQueryCorrectionRetentionDenial> {
        let runtime = self.runtime();
        let originals = runtime
            .runtime
            .product_runtime
            .owner
            .inspection_port()
            .observe_program_activation_history(execution)
            .map_err(WorthQueryCorrectionRetentionDenial::World)?;
        runtime
            .runtime
            .primary_provider
            .expire_optional_correction_material(&originals, coordinates, execution)
    }
}
