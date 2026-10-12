use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryDeclarationEntryContributionCategoryFamily,
    WorthQueryDomainEntryMarker,
};
use crate::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface;
use crate::domain_installation::{
    WorthQueryDomainIdentityDeclaration, WorthQueryDomainIdentityName,
    WorthQueryDomainIdentityNamespace, WorthQueryDomainPackage, WorthQueryDomainSemanticVersion,
};
use crate::runtime::WorthQueryWorkspace;
use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorthQueryDomainCapabilityCertificationDomain;

impl WorthQueryDomainEntryMarker for WorthQueryDomainCapabilityCertificationDomain {
    fn domain_key(&self) -> &'static str {
        "worth.spatial"
    }

    fn display_name(&self) -> &'static str {
        "Domain Capability Certification"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[]
    }
}

/// Owns the runtime that minted the certification contribution surface.
///
/// Certification deliberately exercises the same package/install/lookup path
/// as an external consumer. Keeping the workspace alive prevents the fixture
/// from turning an installed handle into a detached synthetic token.
pub(crate) struct WorthQueryDomainCapabilityCertificationInstallation {
    _workspace: WorthQueryWorkspace,
    contributions: WorthQueryInstalledDomainContributionSurface,
}

impl WorthQueryDomainCapabilityCertificationInstallation {
    pub(crate) fn contributions(&self) -> &WorthQueryInstalledDomainContributionSurface {
        &self.contributions
    }

    #[cfg(test)]
    pub(crate) fn workspace(&self) -> &WorthQueryWorkspace {
        &self._workspace
    }
}

pub(crate) fn install_domain_capability_certification(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDomainCapabilityCertificationInstallation {
    let marker = WorthQueryDomainCapabilityCertificationDomain;
    let package = WorthQueryDomainPackage::declare(
        marker,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("worth")
                .expect("static certification namespace must admit"),
            WorthQueryDomainIdentityName::new("spatial")
                .expect("static certification domain name must admit"),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::Admission)
    .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability)
    .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::InvariantCapability)
    .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview)
    .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::ContinuityLineage)
    .permits_contribution(
        WorthQueryDeclarationEntryContributionCategoryFamily::ConsequenceAftermath,
    )
    .permits_contribution(
        WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection,
    );
    let schema = WorthQueryTestBackendSchema::single_collection("CertificationEntity")
        .aspect_contract(certification_identity_contract())
        .expect("static certification contract must admit")
        .aspect("identity.id", "identity.id")
        .expect("static certification schema must admit");
    let workspace = in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .workspace(execution, "domain-capability-certification")
        .expect("certification domain must install through a real runtime");
    let handle = workspace
        .domain(marker)
        .expect("certification domain must be discoverable after installation");
    let contributions = handle
        .contributions_in(&workspace)
        .expect("certification contributions require the owning runtime");

    WorthQueryDomainCapabilityCertificationInstallation {
        _workspace: workspace,
        contributions,
    }
}

fn certification_identity_contract() -> AspectContract {
    required_string_field_contract("identity", AspectIdentity(0x5751_1701), "id")
}

fn required_string_field_contract(
    aspect: &'static str,
    identity: AspectIdentity,
    field: &'static str,
) -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new(field).expect("static certification field must admit"),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("required certification field law must be coherent");
    AspectContract::struct_aspect(
        AspectKey::new(aspect).expect("static certification aspect must admit"),
        identity,
        AspectContractRevision(1),
        StructAspectShape::new([field]).expect("certification fields must be unique"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certification_surface_is_minted_by_its_live_installed_runtime() {
        let host_request = crate::test_execution_policy::serial_request();
        let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let installation = install_domain_capability_certification(request_execution);
        let handle = installation
            .workspace()
            .domain(WorthQueryDomainCapabilityCertificationDomain)
            .expect("installed certification handle should remain available");
        let rebound = handle
            .contributions_in(installation.workspace())
            .expect("owning workspace should validate its installed handle");

        assert_eq!(
            rebound.authority_identity(),
            installation.contributions().authority_identity()
        );
    }
}
