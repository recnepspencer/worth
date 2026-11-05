//! Moved whole from `domain_computation::primary_graph::application_installation::program_admission`: the items below and every impl they own.
use std::sync::Arc;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::WorthQueryProgramSupportRoster;

/// The complete, immutable program support one installation admitted, together
/// with the program its first occurrence activates.
pub(in crate::domain_computation::primary_graph) struct WorthQueryAdmittedProgramSupport<Schema> {
    pub(in crate::domain_computation::primary_graph) roster:
        Arc<WorthQueryProgramSupportRoster<Schema>>,
    /// Exact initial and rostered program declarations used to check configured owners before bootstrap.
    pub(in crate::domain_computation::primary_graph) features:
        Vec<worth_query_declaration::facade::application_program::ApplicationFeatureDeclaration>,
    pub(in crate::domain_computation::primary_graph) initial_revision: ApplicationProgramRevision,
}
