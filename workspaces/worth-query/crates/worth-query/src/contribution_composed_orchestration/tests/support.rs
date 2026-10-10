use std::marker::PhantomData;

use worth_foundational::{
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalProfileSet,
    FoundationalProfileSetInput, ObservationActivationProfile, RetentionDeliveryProfile,
    SupportPostureProfile,
};

use crate::application::{
    WorthQueryBridgeContinuationAuthority, WorthQueryCapabilityFamily,
    WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDeclarationCanonicalEntry,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput,
    WorthQueryDeclarationLegalityContract, WorthQueryDeclarationProgressionContract,
    WorthQueryDeclarationRouteContract, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryNeighborhoodCapableGrouping,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ContributionDomain;

impl WorthQueryDomainEntryMarker for ContributionDomain {
    fn domain_key(&self) -> &'static str {
        "test.contribution.composed.domain"
    }

    fn display_name(&self) -> &'static str {
        "ContributionComposedDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ContributionWorld(pub(super) &'static str);

impl WorthQueryDomainOperatingContext<ContributionDomain> for ContributionWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[
            WorthQueryCapabilityFamily::HistoricalEvaluation,
            WorthQueryCapabilityFamily::WorkflowOrchestration,
            WorthQueryCapabilityFamily::PreviewSession,
            WorthQueryCapabilityFamily::QueryComposition,
        ]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[
            WorthQueryConfigSectionFamily::Query,
            WorthQueryConfigSectionFamily::Relational,
            WorthQueryConfigSectionFamily::RuntimeBridge,
            WorthQueryConfigSectionFamily::Signal,
        ]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { format!("contribution-world-{}", self.0) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ContributionFamily;

impl WorthQueryDeclarationFamilyMarker<ContributionDomain> for ContributionFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = crate::application::WorthQuerySignalNotCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "ContributionFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.face", "selection.active_face"],
            &[],
            &[],
            &[],
            &[],
        )
    }

    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        WorthQueryDeclarationAspectCoverage::from_slices(
            &["selection.face", "selection.active_face"],
            &[],
            &[],
        )
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DeferredContributionFamily;

impl WorthQueryDeclarationFamilyMarker<ContributionDomain> for DeferredContributionFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = crate::application::WorthQuerySignalNotCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "DeferredContributionFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        ContributionFamily::aspect_contract()
    }

    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        ContributionFamily::aspect_coverage()
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }

    fn progression_contract(
        _handle_identity_digest: &str,
        _operating_context_identity_digest: &str,
    ) -> WorthQueryDeclarationProgressionContract {
        WorthQueryDeclarationProgressionContract::deferred_support()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ContributionInput {
    id: &'static str,
    _marker: PhantomData<ContributionFamily>,
}

impl ContributionInput {
    pub(super) fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<ContributionDomain> for ContributionInput {
    type Family = ContributionFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DeferredContributionInput {
    id: &'static str,
    _marker: PhantomData<DeferredContributionFamily>,
}

impl DeferredContributionInput {
    pub(super) fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<ContributionDomain> for DeferredContributionInput {
    type Family = DeferredContributionFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

pub(super) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    ContributionDomain,
    ContributionWorld,
> {
    crate::application::domain_test_support::installed_declaration_context_with_contributions(
        request_execution, ContributionDomain,
        ContributionWorld("main"),
        [
            crate::application::domain_test_support::family::<ContributionDomain, ContributionFamily>(
            ),
            crate::application::domain_test_support::family::<
                ContributionDomain,
                DeferredContributionFamily,
            >(),
        ],
        [
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::Admission,
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability,
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection,
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview,
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::ContinuityLineage,
        ],
    )
}

pub(super) fn admitted_handle_without_contributions(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    ContributionDomain,
    ContributionWorld,
> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        ContributionDomain,
        ContributionWorld("main"),
        [
            crate::application::domain_test_support::family::<ContributionDomain, ContributionFamily>(
            ),
            crate::application::domain_test_support::family::<
                ContributionDomain,
                DeferredContributionFamily,
            >(),
        ],
    )
}

pub(super) fn target_for_envelope(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        ContributionDomain,
        ContributionWorld,
    >,
    id: &'static str,
) -> crate::domain_capabilities::WorthQueryInstalledDeclarationContributionTarget {
    let progressed = handle
        .declare_review_and_progress(ContributionInput::new(id))
        .unwrap_or_else(|_| panic!("expected progressed declaration"));
    handle.contribution_target(progressed.canonical_declaration())
}

pub(super) fn standard_profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: ExecutionObjectiveProfile::Balanced,
        observation_activation: ObservationActivationProfile::Continuous,
    })
    .unwrap()
}
