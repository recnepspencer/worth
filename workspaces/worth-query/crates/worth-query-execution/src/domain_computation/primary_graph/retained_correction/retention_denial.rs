//! Causes from the actual owners participating in correction retention.
#[derive(Debug)]
pub enum WorthQueryCorrectionRetentionDenial {
    Material(crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialDenial),
    ProgramSupport(crate::domain_computation::primary_graph::program_occurrence::support_lifecycle::journal::QueryProgramSupportDenial),
    World(worth_runtime_world::facade::RuntimeWorldServiceDenial<worth_runtime_world::facade::WorldReadmissionDenial>),
}
impl From<crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialDenial> for WorthQueryCorrectionRetentionDenial {
    fn from(cause: crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialDenial) -> Self { Self::Material(cause) }
}
impl From<crate::domain_computation::primary_graph::program_occurrence::support_lifecycle::journal::QueryProgramSupportDenial> for WorthQueryCorrectionRetentionDenial {
    fn from(cause: crate::domain_computation::primary_graph::program_occurrence::support_lifecycle::journal::QueryProgramSupportDenial) -> Self { Self::ProgramSupport(cause) }
}
