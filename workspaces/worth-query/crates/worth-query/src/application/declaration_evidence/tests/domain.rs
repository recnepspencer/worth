use std::marker::PhantomData;

use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDeclarationCanonicalEntry,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput,
    WorthQueryDeclarationLegalityContract, WorthQueryDeclarationProgressionContract,
    WorthQueryDescriptiveOnlyAuthority, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryNeighborhoodCapableGrouping,
    WorthQueryRelationalTruthAuthority, WorthQuerySignalCompatiblePosture,
    WorthQuerySignalDeferredPosture, WorthQuerySingleOnlyGrouping,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeometryDomain;

impl WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "test.geometry.evidence"
    }

    fn display_name(&self) -> &'static str {
        "GeometryEvidenceDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CollaborativeWorld {
    regime: &'static str,
}

impl CollaborativeWorld {
    pub(super) fn named(regime: &'static str) -> Self {
        Self { regime }
    }
}

impl WorthQueryDomainOperatingContext<GeometryDomain> for CollaborativeWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::HistoricalEvaluation]
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
        let value = { format!("geometry.{}", self.regime) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

macro_rules! declare_family {
    ($name:ident, $family_key:literal, $authority:ty, $signal:ty, $grouped:ty, $legality:expr, $progression:expr) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(super) struct $name;

        impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for $name {
            type PrimaryAuthority = $authority;
            type SignalCompatibility = $signal;
            type GroupedPosture = $grouped;

            fn semantic_family_key() -> &'static str {
                $family_key
            }

            fn legality_contract() -> WorthQueryDeclarationLegalityContract {
                $legality
            }

            fn progression_contract(
                _handle_identity_digest: &str,
                _operating_context_identity_digest: &str,
            ) -> WorthQueryDeclarationProgressionContract {
                $progression
            }
        }
    };
}

declare_family!(
    LegalFamily,
    "split-edge-legal",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationLegalityContract::receipt_hot_boundary(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
declare_family!(
    AdmittedFamily,
    "split-edge",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
declare_family!(
    DeferredFamily,
    "split-edge-deferred",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::deferred_support()
);
declare_family!(
    DeniedFamily,
    "split-edge-denied",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::denied_boundary()
);
declare_family!(
    StaleFamily,
    "split-edge-stale",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::stale_readable()
);
declare_family!(
    FailedFamily,
    "split-edge-failed",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::failed_transition()
);
declare_family!(
    DescriptiveDeferredSignalFamily,
    "split-edge-descriptive-deferred-signal",
    WorthQueryDescriptiveOnlyAuthority,
    WorthQuerySignalDeferredPosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationLegalityContract::receipt_hot_boundary(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AspectRichFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for AspectRichFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "split-edge-aspect-rich"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
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
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "split-edge-conflicting-aspect"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
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
pub(super) struct IllegalRoleFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for IllegalRoleFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "split-edge-illegal-role"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::unsupported_boundary()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RebindRequiredFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for RebindRequiredFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "split-edge-rebind-required"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn progression_contract(
        _handle_identity_digest: &str,
        _operating_context_identity_digest: &str,
    ) -> WorthQueryDeclarationProgressionContract {
        WorthQueryDeclarationProgressionContract::rebind_required()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Declaration<F> {
    edge_ref: &'static str,
    _family: PhantomData<F>,
}

impl<F> Declaration<F> {
    pub(super) fn new(edge_ref: &'static str) -> Self {
        Self {
            edge_ref,
            _family: PhantomData,
        }
    }
}

macro_rules! impl_input {
    ($($family:ty),+ $(,)?) => {
        $(
            impl WorthQueryDeclarationInput<GeometryDomain> for Declaration<$family> {
                type Family = $family;

                fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
                    vec![WorthQueryDeclarationCanonicalEntry::text("edge_ref", self.edge_ref)]
                }
            }
        )+
    };
}

impl_input!(
    LegalFamily,
    AdmittedFamily,
    DeferredFamily,
    DeniedFamily,
    StaleFamily,
    FailedFamily,
    DescriptiveDeferredSignalFamily,
    AspectRichFamily,
    ConflictingAspectFamily,
    IllegalRoleFamily,
    RebindRequiredFamily,
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
        CollaborativeWorld::named(regime),
        [
            crate::application::domain_test_support::family::<GeometryDomain, LegalFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, AdmittedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, DeniedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, StaleFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, FailedFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                DescriptiveDeferredSignalFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, AspectRichFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                ConflictingAspectFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, IllegalRoleFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, RebindRequiredFamily>(
            ),
        ],
    )
}

pub(super) fn digest_text(digest: &worth_foundational::facade::CanonicalDerivedDigest) -> String {
    format!("{digest:?}")
}
