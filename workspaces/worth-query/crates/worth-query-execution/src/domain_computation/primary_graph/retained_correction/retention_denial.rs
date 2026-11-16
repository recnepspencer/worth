//! Causes from the actual owners participating in correction retention.
#[derive(Debug)]
pub enum WorthQueryCorrectionRetentionDenial {
    Material(crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialDenial),
    ProgramSupport(crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial),
    World(worth_runtime_world::facade::RuntimeWorldServiceDenial<worth_runtime_world::facade::WorldReadmissionDenial>),
}
impl From<crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialDenial> for WorthQueryCorrectionRetentionDenial {
    fn from(cause: crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialDenial) -> Self { Self::Material(cause) }
}
impl From<crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial>
    for WorthQueryCorrectionRetentionDenial
{
    fn from(
        cause: crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
    ) -> Self {
        Self::ProgramSupport(cause)
    }
}
