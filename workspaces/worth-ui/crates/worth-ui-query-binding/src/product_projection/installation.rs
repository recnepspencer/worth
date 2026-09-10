use worth_query::facade::{domain, runtime};

use super::{
    configure_product_projection_backend, evaluate_product_projection_support,
    platform_pulse_bridge, shared_source_state, SharedSourceState,
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
    bridge: worth_runtime_bridge::facade::RuntimeBridge,
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
        let bridge =
            platform_pulse_bridge().map_err(WorthUiScalarProjectionInstallationError::Bridge)?;
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
        super::WorthUiScalarProjectionInstallation::open(workspace, self.bridge, self.source)
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
    bridge: worth_runtime_bridge::facade::RuntimeBridge,
) -> Result<runtime::WorthQueryRuntimeBuilder, WorthUiScalarProjectionInstallationError> {
    let builder = runtime::WorthQueryRuntime::builder(product_world_resources())
        .domain_package(crate::worth_ui_domain_package())
        .map_err(|error| WorthUiScalarProjectionInstallationError::DomainPackage(Box::new(error)))?
        .domain_package(crate::presentation_async::worth_ui_presentation_async_domain_package())
        .map_err(|error| {
            WorthUiScalarProjectionInstallationError::DomainPackage(Box::new(error))
        })?;
    let builder = crate::install_worth_ui_operation_executors(builder)
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .map_err(|error| {
            WorthUiScalarProjectionInstallationError::AspectContract(Box::new(error))
        })?;
    let builder = crate::presentation_async::install_worth_ui_presentation_async_runtime(builder)
        .map_err(|error| {
        WorthUiScalarProjectionInstallationError::AspectContract(Box::new(error))
    })?;
    let builder = configure_product_projection_backend(builder, bridge, source)
        .map_err(WorthUiScalarProjectionInstallationError::SourceLifecycle)?;
    Ok(projection_consumer_support(builder))
}

fn product_world_resources() -> runtime::WorthQueryProductWorldResources {
    runtime::WorthQueryProductWorldResources::install(
        runtime::RuntimeWorldBudgetInstallation {
            branches: runtime::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 128,
            },
            history: runtime::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1_024,
                history_metadata_bytes: 16 * 1024 * 1024,
            },
            observations: runtime::RuntimeWorldObservationBudgetInstallation {
                active_observations: 512,
            },
            publication: runtime::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 128,
            },
            recovery: runtime::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 128,
                retained_partial_metadata_bytes: 16 * 1024 * 1024,
            },
            retention: runtime::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 1_024,
                in_flight_pin_acquisition_reservations: 256,
            },
            custody: runtime::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 256,
            },
        },
        runtime::WorthQueryProductWorldClock::start(),
    )
    .expect("the UI Product World resources are valid")
}

fn projection_consumer_support(
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
