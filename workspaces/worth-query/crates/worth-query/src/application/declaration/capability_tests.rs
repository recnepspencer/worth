use super::{WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationInput};
use crate::application::{
    assert_declaration_aspect_projections, WorthQueryCapabilityFamily,
    WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDeclarationAspectFit,
    WorthQueryDeclarationCapabilityStatus, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationLegalityContract, WorthQueryDeclaredFamilyChecked,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryNeighborhoodCapableGrouping, WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GeometryDomain;

impl WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "test.geometry.capability"
    }

    fn display_name(&self) -> &'static str {
        "GeometryCapabilityDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct QueryOnlyWorld;

impl WorthQueryDomainOperatingContext<GeometryDomain> for QueryOnlyWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryContext]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[WorthQueryConfigSectionFamily::Query]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { "query-only-world".to_string() };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DurableFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for DurableFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "durable-family"
    }

    fn required_capability_families() -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::DurableArtifacts]
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_edge"],
            &["selection.active_edge"],
            &["selection.active_edge"],
            &[],
            &["selection.material_edit"],
        )
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HistoricalFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for HistoricalFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "historical-family"
    }

    fn required_capability_families() -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::HistoricalEvaluation]
    }

    fn required_config_sections() -> &'static [WorthQueryConfigSectionFamily] {
        &[WorthQueryConfigSectionFamily::Relational]
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HistoricalDeclaration;

impl WorthQueryDeclarationInput<GeometryDomain> for HistoricalDeclaration {
    type Family = HistoricalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text(
            "edge_ref", "edge:99",
        )]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MaskedCoverageFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for MaskedCoverageFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "masked-coverage-family"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_edge"],
            &["selection.active_edge"],
            &["selection.active_edge"],
            &[],
            &[],
        )
    }

    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        WorthQueryDeclarationAspectCoverage::from_slices(
            &["selection.active_edge"],
            &["selection.active_edge"],
            &[],
        )
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
}

fn admitted_query_only_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<GeometryDomain, QueryOnlyWorld>
{
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomain,
        QueryOnlyWorld,
        [
            crate::application::domain_test_support::family::<GeometryDomain, HistoricalFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, MaskedCoverageFamily>(
            ),
        ],
    )
}

#[test]
fn package_admission_rejects_a_deferred_declaration_family() {
    let package =
        crate::application::domain_test_support::domain_package(GeometryDomain).declaration_family(
            crate::application::domain_test_support::family::<GeometryDomain, DurableFamily>(),
        );
    let validated = package.validate().expect("test package should validate");
    let denial = match crate::domain_installation::admit_domain_package(validated) {
        Ok(_) => panic!("a deferred declaration family must not install"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.kind(),
        crate::domain_installation::WorthQueryDomainPackageAdmissionDenialKind::DeferredCapability
    );
    assert_eq!(denial.subject(), "durable_artifacts");
}

#[test]
fn family_support_and_checked_declaration_agree_on_invalid_context_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_query_only_handle(request_execution);
    let support = handle.family_support::<HistoricalFamily>();
    assert_eq!(
        support.declare_status(),
        WorthQueryDeclarationCapabilityStatus::InvalidContext
    );

    match handle.declare_checked(HistoricalDeclaration) {
        WorthQueryDeclaredFamilyChecked::InvalidContext(denial) => {
            assert_eq!(
                denial.capability_status(),
                WorthQueryDeclarationCapabilityStatus::InvalidContext
            );
        }
        other => panic!(
            "expected invalid-context denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn family_support_report_exposes_aspect_contract_alongside_family_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_query_only_handle(request_execution);
    let support = handle.family_support::<MaskedCoverageFamily>();

    assert_declaration_aspect_projections(
        support.aspect_contract().required(),
        &["selection.active_edge"],
    );
    assert_eq!(
        support
            .row(crate::application::WorthQueryDeclarationCapabilityVerb::Declare)
            .expect("declare row should exist")
            .aspect_fit(),
        WorthQueryDeclarationAspectFit::MissingRequired
    );
}

#[test]
fn family_support_report_can_expose_masked_semantic_slices_without_losing_family_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_query_only_handle(request_execution);
    let support = handle.family_support::<MaskedCoverageFamily>();

    assert_eq!(
        support.declare_status(),
        WorthQueryDeclarationCapabilityStatus::Admitted
    );
    assert_declaration_aspect_projections(
        support.aspect_coverage().masked(),
        &["selection.active_edge"],
    );
    assert_eq!(
        support
            .row(crate::application::WorthQueryDeclarationCapabilityVerb::Declare)
            .expect("declare row should exist")
            .aspect_fit(),
        WorthQueryDeclarationAspectFit::MissingRequired
    );
}
