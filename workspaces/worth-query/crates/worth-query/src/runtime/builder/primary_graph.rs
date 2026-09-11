use worth_query_declaration::facade::application_schema::{
    ApplicationPrincipalBindingRef, ApplicationSchema,
};
use worth_query_execution::facade::integration::{
    prepare_primary_graph_product_bridge, prepare_primary_graph_with_relational_runtime,
    publish_primary_graph, retain_primary_graph_integration_handle,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphPublication,
};
use worth_query_installation::facade::{
    ApplicationIdentityScalarValueBinding, WorthQueryExternalPrincipalIdentity,
    WorthQueryInstalledApplicationSchema, WorthQueryPrincipalMappingStatus,
};

use crate::runtime::{
    WorthQueryPrimaryGraphBackendHandle, WorthQueryRuntimeBackend, WorthQueryRuntimeBuilder,
    WorthQueryRuntimeError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrimaryGraphConfigurationDenialKind {
    DuplicateContribution,
    PrincipalBindingRejected,
    GraphInstallationRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphConfigurationDenial {
    kind: WorthQueryPrimaryGraphConfigurationDenialKind,
    subject: String,
}

impl WorthQueryPrimaryGraphConfigurationDenial {
    fn new(
        kind: WorthQueryPrimaryGraphConfigurationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryPrimaryGraphConfigurationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryPrimaryGraphConfigurationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "primary graph configuration denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryPrimaryGraphConfigurationDenial {}

pub struct WorthQueryPrimaryGraphConfiguration<'installation, Schema> {
    installed_schema: &'installation WorthQueryInstalledApplicationSchema<Schema>,
    bootstrap: &'installation mut WorthQueryPrimaryGraphBootstrap<Schema>,
}

impl<Schema> WorthQueryPrimaryGraphConfiguration<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn bind_principal<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &mut self,
        binding: ApplicationPrincipalBindingRef<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        principal_key: WorthQueryApplicationPrincipalKey<Schema, Principal>,
        principal_identity: PrincipalIdentity,
        identity: WorthQueryExternalPrincipalIdentity,
        status: WorthQueryPrincipalMappingStatus,
    ) -> Result<(), WorthQueryPrimaryGraphConfigurationDenial>
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        let installed_binding =
            self.installed_schema
                .principal_binding(binding)
                .map_err(|denial| {
                    WorthQueryPrimaryGraphConfigurationDenial::new(
                        WorthQueryPrimaryGraphConfigurationDenialKind::PrincipalBindingRejected,
                        format!("{:?}: {}", denial.kind(), denial.binding()),
                    )
                })?;
        self.bootstrap
            .bind_principal(
                &installed_binding,
                principal_key,
                principal_identity,
                identity,
                status,
            )
            .map_err(map_graph_denial)
    }
}

pub(super) trait PendingPrimaryGraphInstallation {
    fn install(
        self: Box<Self>,
        execution_runtime: &mut worth_query_execution::facade::runtime::WorthQueryExecutionRuntime,
        installation_authority: &worth_query_execution::facade::runtime::WorthQueryExecutionInstallationAuthority,
        backend: &mut dyn WorthQueryRuntimeBackend,
        product_world_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
    ) -> Result<PendingPrimaryGraphInstallationOutcome, WorthQueryRuntimeError>;
}

pub(super) struct PendingPrimaryGraphInstallationOutcome {
    pub(super) publication: WorthQueryPrimaryGraphPublication,
    pub(super) product_bridge: worth_runtime_bridge::facade::RuntimeBridge,
}

struct TypedPrimaryGraphInstallation<Schema, Configure> {
    configure: Configure,
    _schema: std::marker::PhantomData<fn() -> Schema>,
}

impl<Schema, Configure> PendingPrimaryGraphInstallation
    for TypedPrimaryGraphInstallation<Schema, Configure>
where
    Schema: ApplicationSchema + 'static,
    Configure: FnOnce(
            &mut WorthQueryPrimaryGraphConfiguration<'_, Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphConfigurationDenial>
        + 'static,
{
    fn install(
        self: Box<Self>,
        execution_runtime: &mut worth_query_execution::facade::runtime::WorthQueryExecutionRuntime,
        installation_authority: &worth_query_execution::facade::runtime::WorthQueryExecutionInstallationAuthority,
        backend: &mut dyn WorthQueryRuntimeBackend,
        product_world_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
    ) -> Result<PendingPrimaryGraphInstallationOutcome, WorthQueryRuntimeError> {
        let declaration = Schema::declaration().map_err(|denial| {
            primary_graph_runtime_error("primary_graph_schema_declaration", format!("{denial:?}"))
        })?;
        let installed_schema = execution_runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .map_err(|denial| {
                primary_graph_runtime_error(
                    "primary_graph_schema_binding",
                    format!("{:?}: {}", denial.kind(), denial.subject()),
                )
            })?;
        let transferred = backend
            .surrender_unpublished_primary_graph_runtime()
            .map_err(WorthQueryRuntimeError::Workspace)?;
        let mut bootstrap = prepare_primary_graph_with_relational_runtime(
            installation_authority,
            execution_runtime,
            &installed_schema,
            transferred.into_runtime(),
            product_world_resources,
        )
        .map_err(|denial| {
            primary_graph_runtime_error(
                "primary_graph_bootstrap_preparation",
                format!("{:?}: {}", denial.kind(), denial.subject()),
            )
        })?;
        let TypedPrimaryGraphInstallation { configure, .. } = *self;
        configure(&mut WorthQueryPrimaryGraphConfiguration {
            installed_schema: &installed_schema,
            bootstrap: &mut bootstrap,
        })
        .map_err(|denial| {
            primary_graph_runtime_error("primary_graph_configuration", denial.to_string())
        })?;
        let publication =
            publish_primary_graph(bootstrap, execution_runtime, installation_authority).map_err(
                |denial| {
                    primary_graph_runtime_error(
                        "primary_graph_publication",
                        format!("{:?}: {}", denial.kind(), denial.subject()),
                    )
                },
            )?;
        let integration =
            retain_primary_graph_integration_handle(execution_runtime).ok_or_else(|| {
                primary_graph_runtime_error(
                    "primary_graph_backend_attachment",
                    "published primary graph did not retain an integration handle",
                )
            })?;
        let product_bridge = prepare_primary_graph_product_bridge(&installed_schema, &integration)
            .map_err(|denial| {
                primary_graph_runtime_error(
                    "primary_graph_product_bridge",
                    format!("{:?}: {}", denial.kind(), denial.subject()),
                )
            })?;
        backend
            .attach_primary_graph_runtime(WorthQueryPrimaryGraphBackendHandle::new(integration))
            .map_err(WorthQueryRuntimeError::Workspace)?;
        Ok(PendingPrimaryGraphInstallationOutcome {
            publication,
            product_bridge,
        })
    }
}

impl WorthQueryRuntimeBuilder {
    pub fn application_primary_graph<Schema, Configure>(
        mut self,
        configure: Configure,
    ) -> Result<Self, WorthQueryPrimaryGraphConfigurationDenial>
    where
        Schema: ApplicationSchema + 'static,
        Configure: FnOnce(
                &mut WorthQueryPrimaryGraphConfiguration<'_, Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphConfigurationDenial>
            + 'static,
    {
        if self.pending_primary_graph_installation.is_some() {
            return Err(WorthQueryPrimaryGraphConfigurationDenial::new(
                WorthQueryPrimaryGraphConfigurationDenialKind::DuplicateContribution,
                "one runtime may install exactly one primary logical graph",
            ));
        }
        self.pending_primary_graph_installation = Some(Box::new(TypedPrimaryGraphInstallation::<
            Schema,
            Configure,
        > {
            configure,
            _schema: std::marker::PhantomData,
        }));
        Ok(self)
    }
}

fn map_graph_denial(
    denial: WorthQueryPrimaryGraphInstallationDenial,
) -> WorthQueryPrimaryGraphConfigurationDenial {
    WorthQueryPrimaryGraphConfigurationDenial::new(
        WorthQueryPrimaryGraphConfigurationDenialKind::GraphInstallationRejected,
        format!("{:?}: {}", denial.kind(), denial.subject()),
    )
}

fn primary_graph_runtime_error(
    stage: &'static str,
    message: impl Into<String>,
) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::InvariantRegistration {
        stage,
        message: message.into(),
    }
}
