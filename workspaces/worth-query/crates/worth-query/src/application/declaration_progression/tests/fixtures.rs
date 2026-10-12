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
        "test.geometry.progression"
    }

    fn display_name(&self) -> &'static str {
        "GeometryProgressionDomain"
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
    fn named(regime: &'static str) -> Self {
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
    (
        $name:ident,
        $family_key:literal,
        $authority:ty,
        $signal:ty,
        $grouped:ty,
        $aspect:expr,
        $legality:expr,
        $progression:expr
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(super) struct $name;

        impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for $name {
            type PrimaryAuthority = $authority;
            type SignalCompatibility = $signal;
            type GroupedPosture = $grouped;

            fn semantic_family_key() -> &'static str {
                $family_key
            }

            fn aspect_contract() -> WorthQueryDeclarationAspectContract {
                $aspect
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
    AdmittedFamily,
    "split-edge",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_edge"],
        &["selection.active_edge"],
        &["selection.active_edge"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
declare_family!(
    ReceiptFamily,
    "split-edge-receipt",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_edge"],
        &["selection.active_edge"],
        &["selection.active_edge"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::receipt_hot_boundary(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
declare_family!(
    DeferredFamily,
    "split-edge-deferred",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_edge"],
        &["selection.active_edge"],
        &["selection.active_edge"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::deferred_support()
);
declare_family!(
    DeniedFamily,
    "split-edge-denied",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_edge"],
        &["selection.active_edge"],
        &["selection.active_edge"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::denied_boundary()
);
declare_family!(
    StaleFamily,
    "split-edge-stale",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_edge"],
        &["selection.active_edge"],
        &["selection.active_edge"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::stale_readable()
);
declare_family!(
    FailedFamily,
    "split-edge-failed",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_edge"],
        &["selection.active_edge"],
        &["selection.active_edge"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::failed_transition()
);
declare_family!(
    AlternateAspectFamily,
    "split-edge-alternate-aspect",
    WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_face"],
        &["selection.active_face"],
        &["selection.active_face"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::authoritative_hot_artifact(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);
declare_family!(
    DescriptiveDeferredSignalFamily,
    "split-edge-descriptive-deferred-signal",
    WorthQueryDescriptiveOnlyAuthority,
    WorthQuerySignalDeferredPosture,
    WorthQuerySingleOnlyGrouping,
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.active_edge"],
        &["selection.active_edge"],
        &["selection.active_edge"],
        &[],
        &[]
    ),
    WorthQueryDeclarationLegalityContract::receipt_hot_boundary(),
    WorthQueryDeclarationProgressionContract::admitted_current()
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MaskedCoverageFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for MaskedCoverageFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "split-edge-masked-coverage"
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

    fn progression_contract(
        _handle_identity_digest: &str,
        _operating_context_identity_digest: &str,
    ) -> WorthQueryDeclarationProgressionContract {
        WorthQueryDeclarationProgressionContract::admitted_current()
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

macro_rules! impl_declaration_input {
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

impl_declaration_input!(
    AdmittedFamily,
    AlternateAspectFamily,
    ReceiptFamily,
    DeferredFamily,
    DeniedFamily,
    StaleFamily,
    FailedFamily,
    DescriptiveDeferredSignalFamily,
    MaskedCoverageFamily,
    RebindRequiredFamily,
);

#[path = "fixtures/admitted_handle.rs"]
mod admitted_handle;
pub(super) use admitted_handle::admitted_handle;

pub(super) fn legal<F>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        GeometryDomain,
        CollaborativeWorld,
    >,
    declaration: Declaration<F>,
) -> crate::application::WorthQueryDeclarationLegalityEvidence<GeometryDomain, Declaration<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    Declaration<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    handle
        .declare_and_review(declaration)
        .unwrap_or_else(|_| panic!("legality review should pass"))
}

pub(super) fn progressed<F>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        GeometryDomain,
        CollaborativeWorld,
    >,
    declaration: Declaration<F>,
) -> crate::application::WorthQueryAdmittedDeclarationProgression<GeometryDomain, Declaration<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    Declaration<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    handle
        .declare_review_and_progress(declaration)
        .unwrap_or_else(|_| panic!("progression should admit"))
}
