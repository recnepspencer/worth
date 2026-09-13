use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration,
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphPublication;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledApplicationSchemaDenial,
};

use super::WorthQueryRuntime;

impl WorthQueryRuntime {
    /// Returns installation evidence for the execution-owned primary graph.
    ///
    /// The receipt is descriptive. It does not expose graph mutation authority
    /// or raw Relational access.
    pub fn primary_graph_publication(&self) -> Option<&WorthQueryPrimaryGraphPublication> {
        self.primary_graph_publication.as_ref()
    }

    /// Binds a typed schema declaration to this runtime's exact installed
    /// package generation.
    pub fn installed_application_schema<Schema>(
        &self,
        declaration: ApplicationSchemaDeclaration<Schema>,
    ) -> Result<
        WorthQueryInstalledApplicationSchema<Schema>,
        WorthQueryInstalledApplicationSchemaDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        self.execution_runtime
            .installed_packages()
            .bind_application_schema(declaration)
    }
}
