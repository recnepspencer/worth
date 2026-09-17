use worth_query::facade::{domain, runtime};

use super::{
    configure_product_projection_backend, evaluate_product_projection_support, shared_source_state,
    SharedSourceState,
};

pub struct WorthUiQueryHostInstallationRequest {
    inner: runtime::WorthQueryHostRuntimeInstallationRequest,
}

impl WorthUiQueryHostInstallationRequest {
    pub fn generation(&self) -> runtime::WorthQueryInstallationGeneration {
        self.inner.generation()
    }

    pub fn into_packages(self) -> Vec<runtime::WorthQueryAdmittedPortableDomainPackage> {
        self.inner.into_packages()
    }
}

pub struct WorthUiScalarProjectionHostCompletion {
    inner: runtime::WorthQueryHostRuntimeInstallationCompletion,
    bridge: std::sync::Arc<std::sync::OnceLock<worth_runtime_bridge::facade::RuntimeBridge>>,
    source: SharedSourceState,
}

pub struct WorthUiScalarProjectionHostPlan {
    request: WorthUiQueryHostInstallationRequest,
    completion: WorthUiScalarProjectionHostCompletion,
}

impl WorthUiScalarProjectionHostPlan {
    #[allow(
        clippy::result_large_err,
        reason = "cold host installation preserves exact Query failure topology"
    )]
    pub fn prepare() -> Result<Self, WorthUiScalarProjectionInstallationError> {
        let source = shared_source_state();
        let bridge = std::sync::Arc::new(std::sync::OnceLock::new());
        let builder = projection_runtime_builder(source.clone(), bridge.clone())?;
        let plan = builder.prepare_host_installation();
        let (request, completion) = plan.into_parts();
        Ok(Self {
            request: WorthUiQueryHostInstallationRequest { inner: request },
            completion: WorthUiScalarProjectionHostCompletion {
                inner: completion,
                bridge,
                source,
            },
        })
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthUiQueryHostInstallationRequest,
        WorthUiScalarProjectionHostCompletion,
    ) {
        (self.request, self.completion)
    }

    #[cfg(feature = "certification-construction")]
    pub fn install_for_certification(
        self,
    ) -> Result<super::WorthUiScalarProjectionInstallation, WorthUiScalarProjectionInstallationError>
    {
        let (request, completion) = self.into_parts();
        let installation =
            worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
                .install(request.generation(), request.into_packages())
                .map_err(|error| {
                    WorthUiScalarProjectionInstallationError::SourceLifecycle(format!("{error:?}"))
                })?;
        completion.complete(installation)
    }
}

impl WorthUiScalarProjectionHostCompletion {
    #[allow(
        clippy::result_large_err,
        reason = "cold host completion preserves exact Query failure topology"
    )]
    pub fn complete(
        self,
        installation: runtime::WorthQueryExecutionRuntimeInstallation,
    ) -> Result<super::WorthUiScalarProjectionInstallation, WorthUiScalarProjectionInstallationError>
    {
        let runtime = self.inner.complete(installation).map_err(|error| {
            WorthUiScalarProjectionInstallationError::RuntimeCompletion(Box::new(error))
        })?;
        let workspace = runtime
            .workspace("worth-ui-platform-pulse")
            .map_err(|error| WorthUiScalarProjectionInstallationError::Runtime(Box::new(error)))?;
        evaluate_product_projection_support(&workspace).map_err(|error| {
            WorthUiScalarProjectionInstallationError::SourceLifecycle(format!(
                "Query support pin denied product projection installation: {error}"
            ))
        })?;
        let bridge = self
            .bridge
            .get()
            .expect("completed Query installation owns the product Bridge")
            .clone();
        super::WorthUiScalarProjectionInstallation::open(workspace, bridge, self.source)
    }
}

#[derive(Debug)]
pub enum WorthUiScalarProjectionInstallationError {
    AspectContract(Box<runtime::WorthQueryAspectContractRegistrationDenial>),
    Bridge(String),
    DomainPackage(Box<domain::WorthQueryDomainPackageInstallationError>),
    Runtime(Box<runtime::WorthQueryRuntimeError>),
    RuntimeCompletion(Box<runtime::WorthQueryHostRuntimeCompletionError>),
    SourceLifecycle(String),
}

pub(crate) fn projection_runtime_builder(
    source: SharedSourceState,
    bridge: std::sync::Arc<std::sync::OnceLock<worth_runtime_bridge::facade::RuntimeBridge>>,
) -> Result<runtime::WorthQueryRuntimeBuilder, WorthUiScalarProjectionInstallationError> {
    let builder = runtime::WorthQueryRuntime::builder(
        crate::query_runtime_resources::ui_product_world_resources(),
    )
    .domain_package(crate::worth_ui_domain_package())
    .map_err(|error| WorthUiScalarProjectionInstallationError::DomainPackage(Box::new(error)))?;
    let builder = crate::install_worth_ui_operation_executors(builder)
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .map_err(|error| WorthUiScalarProjectionInstallationError::AspectContract(Box::new(error)))?
        .conditional_execution_resources(status_query_conditional_resources());
    let builder = configure_product_projection_backend(builder, bridge, source);
    Ok(projection_consumer_support(builder))
}

fn status_query_conditional_resources() -> runtime::WorthQueryConditionalExecutionResources {
    runtime::WorthQueryConditionalExecutionResources::new(
        runtime::WorthQueryConditionalEvaluationCacheBudget::bounded(1, 12 * 1024)
            .expect("the status query has a positive cache budget"),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: 4 * 1024 * 1024,
            maximum_attempt_visits: 4 * 1024,
        },
    )
}

pub(crate) fn projection_consumer_support(
    builder: runtime::WorthQueryRuntimeBuilder,
) -> runtime::WorthQueryRuntimeBuilder {
    use domain::{
        WorthQueryConsumerSupportDimension as Dimension,
        WorthQueryConsumerSupportPosture as Posture,
    };

    [
        Dimension::Live,
        Dimension::ProjectionConsumption,
        Dimension::Sharing,
        Dimension::Invalidation,
        Dimension::AsyncResultState,
        Dimension::Recovery,
        Dimension::DependencyImpact,
        Dimension::ConditionalEvaluation,
        Dimension::ConditionalComparator,
        Dimension::ConditionalTrigger,
    ]
    .into_iter()
    .fold(builder, |builder, dimension| {
        builder.consumer_support_posture(dimension, Posture::Supported)
    })
}
