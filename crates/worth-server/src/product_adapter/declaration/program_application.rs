//! Native program ownership enters the same readiness path as raw installed applications.
use super::WorthServerProductOperationDeclaration;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    declaration::application_schema::ApplicationSchema,
};

impl WorthServerProductOperationDeclaration {
    /// Retains the authentic installed program for read-only readiness inspection.
    /// No runtime is rebuilt or extracted, and readiness grants no execution authority.
    /// ```
    /// use worth_query_host::facade::{application_installation::WorthQueryProgramApplicationRuntime,
    ///     declaration::application_schema::ApplicationSchema};
    /// use worth_server::WorthServerProductOperationDeclaration;
    /// fn attach<S: ApplicationSchema + Send + Sync + 'static, P: Send + Sync + 'static>(
    ///     declaration: WorthServerProductOperationDeclaration,
    ///     program: &WorthQueryProgramApplicationRuntime<S, P>,
    /// ) -> WorthServerProductOperationDeclaration {
    ///     declaration.with_program_application(program.clone())
    /// }
    /// ```
    pub fn with_program_application<Schema, Program>(
        mut self,
        application: WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Self
    where
        Schema: ApplicationSchema + Send + Sync + 'static,
        Program: Send + Sync + 'static,
    {
        self.query_application_readiness = Some(
            super::super::program_application_readiness_provider(application),
        );
        self
    }
}
