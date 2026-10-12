use std::marker::PhantomData;

#[path = "authority_rich.rs"]
mod authority_rich;

use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDeclarationBridgeContinuationContract,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationProgressionContract, WorthQueryDeclarationRouteContract,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryNeighborhoodCapableGrouping, WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture, WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
};

pub(super) use authority_rich::AuthorityRichFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeometryDomain;

impl WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "test.geometry.entry-orchestration"
    }
    fn display_name(&self) -> &'static str {
        "GeometryEntryOrchestrationDomain"
    }
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CollaborativeWorld(pub &'static str);

impl WorthQueryDomainOperatingContext<GeometryDomain> for CollaborativeWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[
            WorthQueryCapabilityFamily::HistoricalEvaluation,
            WorthQueryCapabilityFamily::WorkflowOrchestration,
            WorthQueryCapabilityFamily::PreviewSession,
            WorthQueryCapabilityFamily::QueryComposition,
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
        let value = { format!("entry.orchestration.{}", self.0) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

macro_rules! define_family {
    ($name:ident, $authority:ty, $signal:ty, $grouped:ty, $route:expr, $progression:expr) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(super) struct $name;

        impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for $name {
            type PrimaryAuthority = $authority;
            type SignalCompatibility = $signal;
            type GroupedPosture = $grouped;

            fn semantic_family_key() -> &'static str {
                stringify!($name)
            }
            fn legality_contract() -> WorthQueryDeclarationLegalityContract {
                WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
            }
            fn route_contract() -> WorthQueryDeclarationRouteContract {
                $route
            }
            fn progression_contract(
                _handle_identity_digest: &str,
                operating_context_identity_digest: &str,
            ) -> WorthQueryDeclarationProgressionContract {
                let _ = operating_context_identity_digest;
                $progression
            }
            fn bridge_continuation_contract(
            ) -> Option<WorthQueryDeclarationBridgeContinuationContract> {
                None
            }
        }
    };
}

define_family!(
    AdmittedFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationRouteContract::relational_only(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
define_family!(
    DeferredFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::relational_only(),
    WorthQueryDeclarationProgressionContract::deferred_support()
);
define_family!(
    DeniedFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::relational_only(),
    WorthQueryDeclarationProgressionContract::denied_boundary()
);
define_family!(
    StaleFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::relational_only(),
    WorthQueryDeclarationProgressionContract::stale_readable()
);
define_family!(
    FailedFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::relational_only(),
    WorthQueryDeclarationProgressionContract::failed_transition()
);
define_family!(
    ExplicitIntentFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::required_relational_intent(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
define_family!(
    DeferredRouteFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::deferred_auto(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
define_family!(
    UnsupportedReceiptFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::signal_only(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
define_family!(
    ExpensiveAutomationFamily,
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalNotCompatiblePosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationRouteContract::expensive_by_default_for_tests(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AspectRichFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for AspectRichFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalNotCompatiblePosture;
    type GroupedPosture = WorthQuerySingleOnlyGrouping;

    fn semantic_family_key() -> &'static str {
        "AspectRichFamily"
    }
    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::relational_only()
    }
    fn progression_contract(
        _handle_identity_digest: &str,
        _operating_context_identity_digest: &str,
    ) -> WorthQueryDeclarationProgressionContract {
        WorthQueryDeclarationProgressionContract::admitted_current()
    }
    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_edge"],
            &["selection.local_topology"],
            &["selection.material_edit"],
            &["selection.private_authority"],
            &[],
        )
    }
    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        WorthQueryDeclarationAspectCoverage::from_slices(
            &[
                "selection.active_edge",
                "selection.local_topology",
                "selection.material_edit",
                "selection.private_authority",
            ],
            &["selection.private_authority"],
            &[],
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ConflictingAspectFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for ConflictingAspectFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalNotCompatiblePosture;
    type GroupedPosture = WorthQuerySingleOnlyGrouping;

    fn semantic_family_key() -> &'static str {
        "ConflictingAspectFamily"
    }
    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::relational_only()
    }
    fn progression_contract(
        _handle_identity_digest: &str,
        _operating_context_identity_digest: &str,
    ) -> WorthQueryDeclarationProgressionContract {
        WorthQueryDeclarationProgressionContract::admitted_current()
    }
    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_edge"],
            &["selection.local_topology"],
            &["selection.material_edit"],
            &["selection.private_authority"],
            &[],
        )
    }
    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        WorthQueryDeclarationAspectCoverage::from_slices(
            &[
                "selection.active_edge",
                "selection.local_topology",
                "selection.material_edit",
                "selection.private_authority",
            ],
            &["selection.private_authority"],
            &["selection.material_edit"],
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RebindRequiredFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for RebindRequiredFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalNotCompatiblePosture;
    type GroupedPosture = WorthQuerySingleOnlyGrouping;
    fn semantic_family_key() -> &'static str {
        "RebindRequiredFamily"
    }
    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::relational_only()
    }
    fn progression_contract(
        _handle_identity_digest: &str,
        _operating_context_identity_digest: &str,
    ) -> WorthQueryDeclarationProgressionContract {
        WorthQueryDeclarationProgressionContract::rebind_required()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Input<F>(pub &'static str, pub PhantomData<F>);

impl<F> Input<F> {
    pub(super) fn new(edge_ref: &'static str) -> Self {
        Self(edge_ref, PhantomData)
    }
}

macro_rules! impl_input {
    ($($family:ty),+ $(,)?) => {$(
        impl WorthQueryDeclarationInput<GeometryDomain> for Input<$family> {
            type Family = $family;
            fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
                vec![WorthQueryDeclarationCanonicalEntry::text("edge_ref", self.0)]
            }
        }
    )+};
}

impl_input!(
    AdmittedFamily,
    DeferredFamily,
    DeniedFamily,
    StaleFamily,
    FailedFamily,
    ExplicitIntentFamily,
    DeferredRouteFamily,
    UnsupportedReceiptFamily,
    ExpensiveAutomationFamily,
    AspectRichFamily,
    ConflictingAspectFamily,
    RebindRequiredFamily,
    AuthorityRichFamily
);

pub(super) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    GeometryDomain,
    CollaborativeWorld,
> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomain,
        CollaborativeWorld(regime),
        [
            crate::application::domain_test_support::family::<GeometryDomain, AdmittedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, DeniedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, StaleFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, FailedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, ExplicitIntentFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredRouteFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                UnsupportedReceiptFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ExpensiveAutomationFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, AspectRichFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ConflictingAspectFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, RebindRequiredFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, AuthorityRichFamily>(
            ),
        ],
    )
}
