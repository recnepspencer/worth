use std::collections::{HashMap, HashSet};

use worth_query::facade::runtime;

use super::{
    correspondence, WorthUiPresentationAsyncOwner, WorthUiPresentationCorrespondenceIssuer,
};

pub struct WorthUiPresentationAsyncHostPlan {
    request: WorthUiPresentationQueryHostInstallationRequest,
    completion: WorthUiPresentationAsyncHostCompletion,
}

pub struct WorthUiPresentationQueryHostInstallationRequest {
    inner: runtime::WorthQueryHostRuntimeInstallationRequest,
}

pub struct WorthUiPresentationAsyncHostCompletion {
    inner: runtime::WorthQueryHostRuntimeInstallationCompletion,
}

pub struct WorthUiPresentationAsyncInstallation {
    owner: WorthUiPresentationAsyncOwner,
    correspondence: WorthUiPresentationCorrespondenceIssuer,
}

#[derive(Debug)]
pub enum WorthUiPresentationAsyncInstallationError {
    Builder(Box<super::super::super::WorthUiScalarProjectionInstallationError>),
    Completion(Box<runtime::WorthQueryHostRuntimeCompletionError>),
    Runtime(Box<runtime::WorthQueryRuntimeError>),
    OperatingWorld(Box<worth_query::facade::installed::WorthQueryOperatingWorldEntryDenial>),
    MissingOwnedAsyncSource,
}

impl WorthUiPresentationAsyncHostPlan {
    pub fn prepare() -> Result<Self, WorthUiPresentationAsyncInstallationError> {
        let source = crate::product_projection::shared_source_state();
        let bridge = crate::product_projection::platform_pulse_bridge().map_err(|detail| {
            WorthUiPresentationAsyncInstallationError::Builder(Box::new(
                super::super::super::WorthUiScalarProjectionInstallationError::Bridge(detail),
            ))
        })?;
        let builder = crate::product_projection::projection_runtime_builder(source, bridge)
            .map_err(|error| WorthUiPresentationAsyncInstallationError::Builder(Box::new(error)))?;
        let plan = builder.prepare_host_installation();
        let (request, completion) = plan.into_parts();
        Ok(Self {
            request: WorthUiPresentationQueryHostInstallationRequest { inner: request },
            completion: WorthUiPresentationAsyncHostCompletion { inner: completion },
        })
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthUiPresentationQueryHostInstallationRequest,
        WorthUiPresentationAsyncHostCompletion,
    ) {
        (self.request, self.completion)
    }
}

impl WorthUiPresentationQueryHostInstallationRequest {
    pub fn generation(&self) -> runtime::WorthQueryInstallationGeneration {
        self.inner.generation()
    }

    pub fn into_packages(self) -> Vec<runtime::WorthQueryAdmittedPortableDomainPackage> {
        self.inner.into_packages()
    }
}

impl WorthUiPresentationAsyncHostCompletion {
    pub fn complete(
        self,
        installation: runtime::WorthQueryExecutionRuntimeInstallation,
    ) -> Result<WorthUiPresentationAsyncInstallation, WorthUiPresentationAsyncInstallationError>
    {
        let runtime = self.inner.complete(installation).map_err(|error| {
            WorthUiPresentationAsyncInstallationError::Completion(Box::new(error))
        })?;
        let workspace = runtime
            .workspace("worth-ui-mounted-presentation")
            .map_err(|error| WorthUiPresentationAsyncInstallationError::Runtime(Box::new(error)))?;
        let world = workspace.observe_operating_world().map_err(|error| {
            WorthUiPresentationAsyncInstallationError::OperatingWorld(Box::new(error))
        })?;
        let product = world
            .retain_product_branch()
            .map_err(|_| WorthUiPresentationAsyncInstallationError::MissingOwnedAsyncSource)?;
        drop(world);
        let owned_async_source =
            super::super::runtime_bridge::installed_presentation_owned_async_source(&workspace)
                .ok_or(WorthUiPresentationAsyncInstallationError::MissingOwnedAsyncSource)?;
        let (correspondence_authority, correspondence) =
            correspondence::correspondence_authority_pair();
        let owner = WorthUiPresentationAsyncOwner {
            correspondence_authority,
            workspace,
            product,
            owned_async_source,
            next_receipt_nonce: 0,
            pending: HashMap::new(),
            settling: HashMap::new(),
            superseded_pending: HashMap::new(),
            superseded_awaiting_completion: HashMap::new(),
            runtime_cleanups: HashMap::new(),
            unresolved: HashMap::new(),
            terminal_closing: HashMap::new(),
            terminal_closed_resources: 0,
            retained: HashMap::new(),
            current: HashMap::new(),
            active_keys: HashSet::new(),
            transition_trace: Vec::new(),
            transition_trace_overflowed: false,
        };
        Ok(WorthUiPresentationAsyncInstallation {
            owner,
            correspondence,
        })
    }
}

impl WorthUiPresentationAsyncInstallation {
    pub fn into_runtime_parts(
        self,
    ) -> (
        WorthUiPresentationAsyncOwner,
        WorthUiPresentationCorrespondenceIssuer,
    ) {
        (self.owner, self.correspondence)
    }
}
