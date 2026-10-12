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
        "test.geometry.relational-routing"
    }

    fn display_name(&self) -> &'static str {
        "GeometryRelationalRoutingDomain"
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
        ]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[
            WorthQueryConfigSectionFamily::Query,
            WorthQueryConfigSectionFamily::Relational,
        ]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { format!("geometry.relational-routing.{}", self.regime) };
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
            crate::application::domain_test_support::family::<GeometryDomain, RuntimeFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, GroupedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, HistoryFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, StrategyFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, BridgeSourceFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, MixedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, SignalOnlyFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, MixedAuthorityFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, MissingAspectFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ConflictingAspectFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, ExpandedAspectFamily>(
            ),
        ],
    )
}

pub(crate) fn relational_aspect_contract() -> WorthQueryDeclarationAspectContract {
    WorthQueryDeclarationAspectContract::from_slices(
        &[
            "selection.active_face",
            "selection.neighborhood.local_topology",
        ],
        &["selection.material_edit"],
        &["selection.preview.surface"],
        &["selection.private_authority"],
        &["selection.conflicting_preview"],
    )
}

pub(crate) fn relational_aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
    WorthQueryDeclarationAspectCoverage::from_slices(
        &[
            "selection.active_face",
            "selection.neighborhood.local_topology",
            "selection.material_edit",
            "selection.preview.surface",
        ],
        &[],
        &[],
    )
}
