//! Server readiness retains the program's original allocation, not a raw-runtime copy.
use super::query_application_readiness::{
    binding_digest, inspect_application_readiness, WorthServerQueryApplicationReadinessDenial,
    WorthServerQueryApplicationReadinessProvider,
};
use std::sync::Arc;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    declaration::application_schema::ApplicationSchema,
    primary_graph::WorthQueryPrimaryGraphApplicationReadinessSnapshot,
};

struct ProgramReadiness<Schema, Program>(WorthQueryProgramApplicationRuntime<Schema, Program>);

impl<Schema, Program> WorthServerQueryApplicationReadinessProvider
    for ProgramReadiness<Schema, Program>
where
    Schema: ApplicationSchema + Send + Sync + 'static,
    Program: Send + Sync + 'static,
{
    fn provider_name(&self) -> &'static str {
        "query-primary-graph-application"
    }
    fn binding_digest(&self) -> String {
        binding_digest(self.0.runtime())
    }
    fn inspect_application_readiness(
        &self,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationReadinessSnapshot,
        WorthServerQueryApplicationReadinessDenial,
    > {
        inspect_application_readiness(self.0.runtime())
    }
}

pub(super) fn program_application_readiness_provider<Schema, Program>(
    application: WorthQueryProgramApplicationRuntime<Schema, Program>,
) -> Arc<dyn WorthServerQueryApplicationReadinessProvider>
where
    Schema: ApplicationSchema + Send + Sync + 'static,
    Program: Send + Sync + 'static,
{
    Arc::new(ProgramReadiness(application))
}
