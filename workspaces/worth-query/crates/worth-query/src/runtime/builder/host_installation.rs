use worth_query_execution::facade::runtime::WorthQueryExecutionRuntimeInstallation;
use worth_query_installation::facade::{
    WorthQueryAdmittedPortableDomainPackage, WorthQueryInstallationGeneration,
};

use super::WorthQueryRuntimeBuilder;
use crate::runtime::{WorthQueryRuntime, WorthQueryRuntimeError};

/// Move-only request for the exact package set already admitted by a runtime builder.
pub struct WorthQueryHostRuntimeInstallationRequest {
    generation: WorthQueryInstallationGeneration,
    packages: Vec<WorthQueryAdmittedPortableDomainPackage>,
}

impl WorthQueryHostRuntimeInstallationRequest {
    pub fn generation(&self) -> WorthQueryInstallationGeneration {
        self.generation
    }

    pub fn into_packages(self) -> Vec<WorthQueryAdmittedPortableDomainPackage> {
        self.packages
    }
}

/// Move-only authority that can complete only the builder that issued its request.
pub struct WorthQueryHostRuntimeInstallationCompletion {
    builder: WorthQueryRuntimeBuilder,
    generation: WorthQueryInstallationGeneration,
    expected_packages: Vec<WorthQueryAdmittedPortableDomainPackage>,
}

impl WorthQueryHostRuntimeInstallationCompletion {
    pub fn complete(
        mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        installation: WorthQueryExecutionRuntimeInstallation,
    ) -> Result<WorthQueryRuntime, WorthQueryHostRuntimeCompletionError> {
        validate_installation(&installation, self.generation, &self.expected_packages)
            .map_err(WorthQueryHostRuntimeCompletionError::Installation)?;
        self.builder.host_execution_installation = Some(installation);
        self.builder
            .build(request_execution)
            .map_err(WorthQueryHostRuntimeCompletionError::Runtime)
    }
}

/// Compiler-total split between host installation work and its exact completion.
pub struct WorthQueryHostRuntimeInstallationPlan {
    request: WorthQueryHostRuntimeInstallationRequest,
    completion: WorthQueryHostRuntimeInstallationCompletion,
}

impl WorthQueryHostRuntimeInstallationPlan {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryHostRuntimeInstallationRequest,
        WorthQueryHostRuntimeInstallationCompletion,
    ) {
        (self.request, self.completion)
    }
}

impl WorthQueryRuntimeBuilder {
    pub fn prepare_host_installation(self) -> WorthQueryHostRuntimeInstallationPlan {
        let generation = WorthQueryInstallationGeneration::initial();
        let packages = self
            .pending_domain_installations
            .host_installation_packages();
        WorthQueryHostRuntimeInstallationPlan {
            request: WorthQueryHostRuntimeInstallationRequest {
                generation,
                packages: packages.clone(),
            },
            completion: WorthQueryHostRuntimeInstallationCompletion {
                builder: self,
                generation,
                expected_packages: packages,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHostRuntimeInstallationDenialKind {
    GenerationMismatch,
    PackageCountMismatch,
    MissingDomain,
    PackageIdentityMismatch,
    AdmissionIdentityMismatch,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryHostRuntimeInstallationDenial {
    kind: WorthQueryHostRuntimeInstallationDenialKind,
    subject: String,
}

impl WorthQueryHostRuntimeInstallationDenial {
    fn new(kind: WorthQueryHostRuntimeInstallationDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryHostRuntimeInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryHostRuntimeInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "host runtime installation denied {:?} for `{}`",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryHostRuntimeInstallationDenial {}

#[derive(Debug)]
pub enum WorthQueryHostRuntimeCompletionError {
    Installation(WorthQueryHostRuntimeInstallationDenial),
    Runtime(WorthQueryRuntimeError),
}

impl std::fmt::Display for WorthQueryHostRuntimeCompletionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installation(denial) => denial.fmt(formatter),
            Self::Runtime(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for WorthQueryHostRuntimeCompletionError {}

fn validate_installation(
    installation: &WorthQueryExecutionRuntimeInstallation,
    expected_generation: WorthQueryInstallationGeneration,
    expected_packages: &[WorthQueryAdmittedPortableDomainPackage],
) -> Result<(), WorthQueryHostRuntimeInstallationDenial> {
    let runtime = installation.runtime();
    let installed = runtime.installed_packages();
    if installed.generation() != expected_generation {
        return Err(WorthQueryHostRuntimeInstallationDenial::new(
            WorthQueryHostRuntimeInstallationDenialKind::GenerationMismatch,
            installed.generation().ordinal().to_string(),
        ));
    }
    if installed.counters().installed_package_count != expected_packages.len() {
        return Err(WorthQueryHostRuntimeInstallationDenial::new(
            WorthQueryHostRuntimeInstallationDenialKind::PackageCountMismatch,
            installed.counters().installed_package_count.to_string(),
        ));
    }
    for expected in expected_packages {
        let owner = expected.package().domain_identity().owner();
        let actual = installed.domain(owner).map_err(|_| {
            WorthQueryHostRuntimeInstallationDenial::new(
                WorthQueryHostRuntimeInstallationDenialKind::MissingDomain,
                owner,
            )
        })?;
        if actual.package_identity() != expected.package().identity() {
            return Err(WorthQueryHostRuntimeInstallationDenial::new(
                WorthQueryHostRuntimeInstallationDenialKind::PackageIdentityMismatch,
                owner,
            ));
        }
        if actual.admission_identity() != expected.admission_identity() {
            return Err(WorthQueryHostRuntimeInstallationDenial::new(
                WorthQueryHostRuntimeInstallationDenialKind::AdmissionIdentityMismatch,
                owner,
            ));
        }
    }
    Ok(())
}
