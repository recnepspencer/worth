use std::sync::Arc;

use worth_query_host::facade::{declaration::application_schema::ApplicationSchema, primary_graph};

pub(crate) trait WorthServerQueryApplicationReadinessProvider:
    Send + Sync + 'static
{
    fn provider_name(&self) -> &'static str;
    fn binding_digest(&self) -> String;

    fn inspect_application_readiness(
        &self,
    ) -> Result<
        primary_graph::WorthQueryPrimaryGraphApplicationReadinessSnapshot,
        WorthServerQueryApplicationReadinessDenial,
    >;
}

pub(crate) enum WorthServerQueryApplicationReadinessDenial {
    Advancement(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial),
    ProductSelection(primary_graph::WorthQueryProductBranchAdmissionDenial),
    Application(primary_graph::WorthQueryApplicationQueryAdmissionDenial),
}

impl WorthServerQueryApplicationReadinessDenial {
    pub(crate) fn subject(&self) -> String {
        match self {
            Self::Advancement(cause) => cause.to_string(),
            Self::ProductSelection(denial) => {
                format!("current product branch selection: {denial:?}")
            }
            Self::Application(denial) => denial.subject().to_string(),
        }
    }
}

#[derive(Clone)]
struct WorthServerPrimaryGraphApplicationReadinessProvider<Schema> {
    application: Arc<primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>>,
}

impl<Schema> WorthServerPrimaryGraphApplicationReadinessProvider<Schema> {
    fn new(
        application: Arc<primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>>,
    ) -> Self {
        Self { application }
    }
}

impl<Schema> std::fmt::Debug for WorthServerPrimaryGraphApplicationReadinessProvider<Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthServerPrimaryGraphApplicationReadinessProvider")
            .finish_non_exhaustive()
    }
}

impl<Schema> WorthServerQueryApplicationReadinessProvider
    for WorthServerPrimaryGraphApplicationReadinessProvider<Schema>
where
    Schema: ApplicationSchema + Send + Sync + 'static,
{
    fn provider_name(&self) -> &'static str {
        "query-primary-graph-application"
    }

    fn binding_digest(&self) -> String {
        binding_digest(&self.application)
    }

    fn inspect_application_readiness(
        &self,
    ) -> Result<
        primary_graph::WorthQueryPrimaryGraphApplicationReadinessSnapshot,
        WorthServerQueryApplicationReadinessDenial,
    > {
        inspect_application_readiness(&self.application)
    }
}

pub(super) fn binding_digest<Schema: ApplicationSchema>(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
) -> String {
    let binding = application.installed_schema().binding_identity();
    format!(
        "query-primary-graph-application-v1:{}:{}:{}",
        binding.package_identity().render_hex(),
        binding.schema_identity().render_hex(),
        binding.generation()
    )
}

pub(super) fn inspect_application_readiness<Schema: ApplicationSchema>(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
) -> Result<
    primary_graph::WorthQueryPrimaryGraphApplicationReadinessSnapshot,
    WorthServerQueryApplicationReadinessDenial,
> {
    application
        .with_host_advancement(|phase| {
            application
                .on_branch(application.current_world())
                .select(&phase)
                .map_err(WorthServerQueryApplicationReadinessDenial::ProductSelection)?
                .inspect_application_readiness()
                .map_err(WorthServerQueryApplicationReadinessDenial::Application)
        })
        .map_err(WorthServerQueryApplicationReadinessDenial::Advancement)?
}

pub(super) fn primary_graph_application_readiness_provider<Schema>(
    application: Arc<primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>>,
) -> Arc<dyn WorthServerQueryApplicationReadinessProvider>
where
    Schema: ApplicationSchema + Send + Sync + 'static,
{
    Arc::new(WorthServerPrimaryGraphApplicationReadinessProvider::new(
        application,
    ))
}
