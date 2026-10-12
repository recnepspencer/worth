use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::application::{
    WorthQueryBridgeContinuationAuthority, WorthQueryCapabilityFamily,
    WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationRouteContract, WorthQueryDeclarationSignalCompatibilityContract,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryNeighborhoodCapableGrouping, WorthQuerySignalCompatiblePosture,
};
use crate::family_helpers::{
    WorthQueryGeometryActiveFaceSelectionHelperFamily, WorthQueryGeometryNeighborhoodHelperFamily,
};

static COUNTING_GEOMETRY_CANONICALIZATION_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeometryDomain;

impl WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "test.grouped_authoring.geometry"
    }

    fn display_name(&self) -> &'static str {
        "GroupedAuthoringGeometryDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeometryWorld(pub(super) &'static str);

impl WorthQueryDomainOperatingContext<GeometryDomain> for GeometryWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::PreviewSession]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[
            WorthQueryConfigSectionFamily::Query,
            WorthQueryConfigSectionFamily::RuntimeBridge,
            WorthQueryConfigSectionFamily::Signal,
        ]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { format!("grouped-authoring-world-{}", self.0) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeometryFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for GeometryFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "GeometryGroupedFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_face"],
            &["selection.material_preview"],
            &[],
            &[],
            &[],
        )
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::relational_and_bridge()
    }

    fn bridge_continuation_contract(
    ) -> Option<crate::application::WorthQueryDeclarationBridgeContinuationContract> {
        Some(crate::application::WorthQueryDeclarationBridgeContinuationContract::preview_session())
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        Some(WorthQueryDeclarationSignalCompatibilityContract::runtime_derived_execution())
    }
}

impl WorthQueryGeometryActiveFaceSelectionHelperFamily<GeometryDomain> for GeometryFamily {}
impl WorthQueryGeometryNeighborhoodHelperFamily<GeometryDomain> for GeometryFamily {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RequiredIntentGeometryFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for RequiredIntentGeometryFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "GeometryRequiredIntentGroupedFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_face"],
            &["selection.material_preview"],
            &[],
            &[],
            &[],
        )
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::required_relational_intent()
    }

    fn bridge_continuation_contract(
    ) -> Option<crate::application::WorthQueryDeclarationBridgeContinuationContract> {
        Some(crate::application::WorthQueryDeclarationBridgeContinuationContract::preview_session())
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        Some(WorthQueryDeclarationSignalCompatibilityContract::runtime_derived_execution())
    }
}

impl WorthQueryGeometryNeighborhoodHelperFamily<GeometryDomain> for RequiredIntentGeometryFamily {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GeometryInput {
    id: &'static str,
    _marker: PhantomData<GeometryFamily>,
}

impl GeometryInput {
    pub(super) fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<GeometryDomain> for GeometryInput {
    type Family = GeometryFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        if self.id.is_empty() {
            Vec::new()
        } else {
            vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CountingGeometryInput {
    id: &'static str,
    _marker: PhantomData<GeometryFamily>,
}

impl CountingGeometryInput {
    pub(super) fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<GeometryDomain> for CountingGeometryInput {
    type Family = GeometryFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        COUNTING_GEOMETRY_CANONICALIZATION_COUNT.fetch_add(1, Ordering::SeqCst);
        if self.id.is_empty() {
            Vec::new()
        } else {
            vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RequiredIntentGeometryInput {
    id: &'static str,
    _marker: PhantomData<RequiredIntentGeometryFamily>,
}

impl RequiredIntentGeometryInput {
    pub(super) fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<GeometryDomain> for RequiredIntentGeometryInput {
    type Family = RequiredIntentGeometryFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        if self.id.is_empty() {
            Vec::new()
        } else {
            vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
        }
    }
}

pub(super) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>
{
    installed_geometry_context(request_execution, GeometryWorld(world))
}

pub(super) fn admitted_handle_with_shifted_relational_digest(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>
{
    installed_geometry_context(request_execution, GeometryWorld(world))
}

fn installed_geometry_context(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: GeometryWorld,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>
{
    crate::application::domain_test_support::installed_declaration_context_with_contributions(
        request_execution, GeometryDomain,
        context,
        [
            crate::application::domain_test_support::family::<GeometryDomain, GeometryFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                RequiredIntentGeometryFamily,
            >(),
        ],
        [
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability,
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection,
        ],
    )
}

pub(super) fn reset_counting_geometry_canonicalization_count() {
    COUNTING_GEOMETRY_CANONICALIZATION_COUNT.store(0, Ordering::SeqCst);
}

pub(super) fn counting_geometry_canonicalization_count() -> usize {
    COUNTING_GEOMETRY_CANONICALIZATION_COUNT.load(Ordering::SeqCst)
}
