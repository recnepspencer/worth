use super::super::support::*;
use crate::application::WorthQueryDomainEntryMarker;
use crate::domain_installation::{
    WorthQueryDomainIdentityDeclaration, WorthQueryDomainIdentityName,
    WorthQueryDomainIdentityNamespace, WorthQueryDomainPackage, WorthQueryDomainSemanticVersion,
};
use crate::runtime::{
    WorthQueryHostRuntimeCompletionError, WorthQueryHostRuntimeInstallationDenialKind,
};
use worth_query_execution::facade::runtime::WorthQueryExecutionRuntimeInstaller;

#[derive(Clone, Copy, Eq, PartialEq)]
struct HostInstalledDomain;

impl WorthQueryDomainEntryMarker for HostInstalledDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.host-installed"
    }

    fn display_name(&self) -> &'static str {
        "HostInstalledDomain"
    }

    fn required_capability_families(
        &self,
    ) -> &'static [crate::application::WorthQueryCapabilityFamily] {
        &[]
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct ForeignHostDomain;

impl WorthQueryDomainEntryMarker for ForeignHostDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.foreign-host"
    }

    fn display_name(&self) -> &'static str {
        "ForeignHostDomain"
    }

    fn required_capability_families(
        &self,
    ) -> &'static [crate::application::WorthQueryCapabilityFamily] {
        &[]
    }
}

#[test]
fn host_installs_the_exact_builder_admitted_package_set_into_one_runtime() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let builder = complete_backend_from_parts_builder()
        .domain_package(host_package())
        .expect("host package admits")
        .build_backend_from_parts();
    let (request, completion) = builder.prepare_host_installation().into_parts();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .install(request.generation(), request.into_packages())
        .expect("host installs the exact admitted set");
    let runtime = completion
        .complete(request_execution, installation)
        .expect("the issuing builder consumes the host installation");

    assert!(runtime.domain(HostInstalledDomain).is_ok());
}

#[test]
fn host_completion_rejects_missing_packages_before_runtime_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let builder = complete_backend_from_parts_builder()
        .domain_package(host_package())
        .expect("host package admits")
        .build_backend_from_parts();
    let (request, completion) = builder.prepare_host_installation().into_parts();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .install(request.generation(), [])
        .expect("an empty host index is structurally valid");

    assert_installation_denial(
        completion.complete(request_execution, installation),
        WorthQueryHostRuntimeInstallationDenialKind::PackageCountMismatch,
    );
}

#[test]
fn host_completion_rejects_equal_width_package_substitution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let builder = complete_backend_from_parts_builder()
        .domain_package(host_package())
        .expect("host package admits")
        .build_backend_from_parts();
    let (request, completion) = builder.prepare_host_installation().into_parts();
    let generation = request.generation();
    let foreign_builder = complete_backend_from_parts_builder()
        .domain_package(foreign_package())
        .expect("foreign package admits")
        .build_backend_from_parts();
    let (foreign_request, _) = foreign_builder.prepare_host_installation().into_parts();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .install(generation, foreign_request.into_packages())
        .expect("the foreign package set is independently valid");

    assert_installation_denial(
        completion.complete(request_execution, installation),
        WorthQueryHostRuntimeInstallationDenialKind::MissingDomain,
    );
}

#[test]
fn host_completion_rejects_generation_substitution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let builder = complete_backend_from_parts_builder()
        .domain_package(host_package())
        .expect("host package admits")
        .build_backend_from_parts();
    let (request, completion) = builder.prepare_host_installation().into_parts();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .install(request.generation().successor(), request.into_packages())
        .expect("the successor-generation index is independently valid");

    assert_installation_denial(
        completion.complete(request_execution, installation),
        WorthQueryHostRuntimeInstallationDenialKind::GenerationMismatch,
    );
}

fn host_package() -> WorthQueryDomainPackage<HostInstalledDomain> {
    WorthQueryDomainPackage::declare(
        HostInstalledDomain,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            WorthQueryDomainIdentityName::new("host-installed").unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
}

fn foreign_package() -> WorthQueryDomainPackage<ForeignHostDomain> {
    WorthQueryDomainPackage::declare(
        ForeignHostDomain,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            WorthQueryDomainIdentityName::new("foreign-host").unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
}

fn assert_installation_denial(
    result: Result<crate::runtime::WorthQueryRuntime, WorthQueryHostRuntimeCompletionError>,
    expected: WorthQueryHostRuntimeInstallationDenialKind,
) {
    match result {
        Err(WorthQueryHostRuntimeCompletionError::Installation(denial)) => {
            assert_eq!(denial.kind(), expected);
        }
        Err(other) => panic!("expected host installation denial, got {other:?}"),
        Ok(_) => panic!("substituted host installation must not publish a runtime"),
    }
}
