//! Sealed test doors over this module's private members.
use super::*;

impl WorthQuerySupportedProgramRecord {
    #[doc(hidden)]
    pub fn installed_for_test<Schema, Program>( installed: std::sync::Arc<WorthQueryInstalledApplicationProgram<Schema, Program>>, output_source_bindings: &[TypeId], root_graph_types: &[TypeId]) -> Self
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema> + 'static, {
        Self::installed::<Schema, Program>(installed, output_source_bindings, root_graph_types)
    }
}
