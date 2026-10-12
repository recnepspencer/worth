use super::families::*;
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GeometryDomain;

impl WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "test.geometry.bridge-routing"
    }

    fn display_name(&self) -> &'static str {
        "GeometryBridgeRoutingDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GeometryWorld {
    regime: &'static str,
}

impl GeometryWorld {
    pub(crate) fn named(regime: &'static str) -> Self {
        Self { regime }
    }
}

impl WorthQueryDomainOperatingContext<GeometryDomain> for GeometryWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[
            WorthQueryCapabilityFamily::HistoricalEvaluation,
            WorthQueryCapabilityFamily::WorkflowOrchestration,
            WorthQueryCapabilityFamily::PreviewSession,
            WorthQueryCapabilityFamily::LiveQuery,
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
        let value = { format!("geometry.bridge-routing.{}", self.regime) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

pub(crate) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: &'static str,
) -> WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomain,
        GeometryWorld::named(regime),
        [
            crate::application::domain_test_support::family::<GeometryDomain, RuntimeRouteFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, TruthViewCurrentFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                TruthViewHistoricalFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, PreviewSessionFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, PreviewPromotionFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                SubscriptionPreparationFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                WritebackPreparationFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, SignalOnlyFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, MixedAuthorityFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, RelationalOnlyFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, MissingAspectFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ConflictingAspectFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, ExpandedAspectFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                TemporalRuntimeRouteFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                AsyncRuntimeRouteFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                TemporalSignalOnlyFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, AsyncSignalOnlyFamily>(
            ),
        ],
    )
}

pub(crate) fn bridge_aspect_contract() -> WorthQueryDeclarationAspectContract {
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_face", "continuation.preview_ready"],
        &["selection.neighborhood.local_topology"],
        &["continuation.preview.surface"],
        &["continuation.private_branch"],
        &["continuation.conflicting_preview"],
    )
}

pub(crate) fn bridge_aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
    WorthQueryDeclarationAspectCoverage::from_slices(
        &[
            "selection.active_face",
            "continuation.preview_ready",
            "selection.neighborhood.local_topology",
            "continuation.preview.surface",
        ],
        &[],
        &[],
    )
}
