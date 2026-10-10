use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationRouteContract, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
    WorthQueryNeighborhoodCapableGrouping, WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RecoveryDomain;

impl WorthQueryDomainEntryMarker for RecoveryDomain {
    fn domain_key(&self) -> &'static str {
        "test.recovery.boundary"
    }

    fn display_name(&self) -> &'static str {
        "RecoveryBoundaryDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RecoveryWorld {
    regime: &'static str,
}

impl RecoveryWorld {
    pub(super) fn named(regime: &'static str) -> Self {
        Self { regime }
    }
}

impl WorthQueryDomainOperatingContext<RecoveryDomain> for RecoveryWorld {
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
        let value = { format!("recovery.{}", self.regime) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RequiredIntentRouteFamily;

impl WorthQueryDeclarationFamilyMarker<RecoveryDomain> for RequiredIntentRouteFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "RequiredIntentRouteFamily"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::required_relational_intent()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SignalReceiptFamily;

impl WorthQueryDeclarationFamilyMarker<RecoveryDomain> for SignalReceiptFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "SignalReceiptFamily"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::signal_only()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RecoveryInput<F> {
    edge_ref: &'static str,
    _marker: std::marker::PhantomData<F>,
}

impl<F> RecoveryInput<F> {
    pub(super) fn new(edge_ref: &'static str) -> Self {
        Self {
            edge_ref,
            _marker: std::marker::PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<RecoveryDomain> for RecoveryInput<RequiredIntentRouteFamily> {
    type Family = RequiredIntentRouteFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text(
            "edge_ref",
            self.edge_ref,
        )]
    }
}

impl WorthQueryDeclarationInput<RecoveryDomain> for RecoveryInput<SignalReceiptFamily> {
    type Family = SignalReceiptFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text(
            "edge_ref",
            self.edge_ref,
        )]
    }
}

pub(super) fn recovery_admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: &'static str,
) -> WorthQueryInstalledDomainDeclarationContext<RecoveryDomain, RecoveryWorld> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        RecoveryDomain,
        RecoveryWorld::named(regime),
        [
            crate::application::domain_test_support::family::<
                RecoveryDomain,
                RequiredIntentRouteFamily,
            >(),
            crate::application::domain_test_support::family::<RecoveryDomain, SignalReceiptFamily>(
            ),
        ],
    )
}

pub(super) fn standard_aspect_contract() -> WorthQueryDeclarationAspectContract {
    WorthQueryDeclarationAspectContract::from_slices(
        &["selection.face"],
        &["selection.active_face"],
        &[],
        &[],
        &[],
    )
}

pub(super) fn recovery_progressed<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<RecoveryDomain, RecoveryWorld>,
    declaration: RecoveryInput<F>,
) -> crate::application::WorthQueryAdmittedDeclarationProgression<RecoveryDomain, RecoveryInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<RecoveryDomain>,
    RecoveryInput<F>: WorthQueryDeclarationInput<RecoveryDomain>,
{
    handle
        .declare_review_and_progress(declaration)
        .unwrap_or_else(|_| panic!("recovery progression should admit"))
}

pub(super) fn recovery_foundational<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<RecoveryDomain, RecoveryWorld>,
    progression: crate::application::WorthQueryAdmittedDeclarationProgression<
        RecoveryDomain,
        RecoveryInput<F>,
    >,
) -> crate::application::WorthQueryDeclarationFoundationalEvidence<RecoveryDomain, RecoveryInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<RecoveryDomain>,
    RecoveryInput<F>: WorthQueryDeclarationInput<RecoveryDomain>,
{
    handle
        .describe_foundational(
            crate::application::WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progression,
            ),
        )
        .unwrap_or_else(|_| panic!("recovery foundational evidence should materialize"))
}
