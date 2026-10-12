use super::{families::*, future_families::*};
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeometryDomain;

impl WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "test.geometry.signal-compatibility"
    }

    fn display_name(&self) -> &'static str {
        "GeometrySignalCompatibilityDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeometryWorld(pub &'static str);

impl WorthQueryDomainOperatingContext<GeometryDomain> for GeometryWorld {
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
        let value = { format!("signal.compatibility.{}", self.0) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

pub fn handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: &'static str,
) -> WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomain,
        GeometryWorld(regime),
        [
            crate::application::domain_test_support::family::<GeometryDomain, RuntimeFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, HistoricalFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, PreviewFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, IncompatibleFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, MixedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, MissingAspectFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ConflictingAspectFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, ExpandedAspectFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, TemporalRuntimeFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, AsyncRuntimeFamily>(),
        ],
    )
}

pub(crate) fn signal_aspect_contract() -> WorthQueryDeclarationAspectContract {
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_face", "signal.dependency.runtime_inputs"],
        &["selection.neighborhood.local_topology"],
        &["signal.preview.surface"],
        &["signal.private_authority"],
        &["signal.conflicting_dependency"],
    )
}

pub(crate) fn signal_aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
    WorthQueryDeclarationAspectCoverage::from_slices(
        &[
            "selection.active_face",
            "signal.dependency.runtime_inputs",
            "selection.neighborhood.local_topology",
            "signal.preview.surface",
        ],
        &[],
        &[],
    )
}

pub(crate) fn signal_dependency_aspects() -> WorthQueryDeclarationAspectContract {
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_face", "signal.dependency.runtime_inputs"],
        &["selection.neighborhood.local_topology"],
        &[],
        &["signal.private_authority"],
        &["signal.conflicting_dependency"],
    )
}

pub(crate) fn signal_produced_aspects() -> WorthQueryDeclarationAspectContract {
    WorthQueryDeclarationAspectContract::from_slices(
        &["signal.produced.derived_face_preview"],
        &["signal.produced.material_projection"],
        &["signal.produced.preview.surface"],
        &[],
        &[],
    )
}
