//! Home installs the same configured contribution declarations without publishing a memory runtime.
use super::*;
pub(in crate::domain_computation::primary_graph) struct HomeConditionalProgramParts<Schema> {
    pub(in crate::domain_computation::primary_graph) publication:
        ApplicationRuntimePublication<Schema>,
    pub(in crate::domain_computation::primary_graph) bindings:
        Vec<Box<dyn WorthQueryPendingConditionalOperation<Schema>>>,
    pub(in crate::domain_computation::primary_graph) definitions:
        Vec<Box<dyn super::super::PendingDefinitionDeclaration>>,
    pub(in crate::domain_computation::primary_graph) output_readiness:
        Vec<Box<dyn super::super::super::application_contribution::PendingOutputReadiness<Schema>>>,
}
impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema> {
    pub(in crate::domain_computation::primary_graph) fn into_home_program_parts(
        self,
    ) -> HomeConditionalProgramParts<Schema> {
        HomeConditionalProgramParts {
            publication: self.publication,
            bindings: self.bindings,
            definitions: self.definitions,
            output_readiness: self.output_readiness,
        }
    }
}
