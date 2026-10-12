use std::marker::PhantomData;

mod envelope;

use crate::application::{
    review_declaration_legality, worth_query_checked_declaration_signal_compatibility_on_handle,
    WorthQueryAsyncDeclarationClause, WorthQueryAsyncDeclarationSupport,
    WorthQueryAsyncFailurePosture, WorthQueryAsyncLoadingPosture,
    WorthQueryAsyncRequestIdentityPart, WorthQueryAsyncSourceFamily,
    WorthQueryBridgeContinuationAuthority, WorthQueryCapabilityFamily,
    WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDeclarationBridgeContinuationMode,
    WorthQueryDeclarationBridgeContinuationRequest, WorthQueryDeclarationBridgeTruthContext,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationEnvelopeInput,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationFoundationalEvidenceInput,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityChecked,
    WorthQueryDeclarationLegalityContract, WorthQueryDeclarationLegalityInput,
    WorthQueryDeclarationReceiptInput, WorthQueryDeclarationRouteContract,
    WorthQueryDeclarationRoutePlanInput, WorthQueryDeclarationSignalCompatibilityChecked,
    WorthQueryDeclarationSignalCompatibilityContract,
    WorthQueryDeclarationSignalCompatibilityInput,
    WorthQueryDeclarationSignalCompatibilitySupportRow,
    WorthQueryDeclarationSignalCompatibilitySupportStatus, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryDomainOperatingRequirement,
    WorthQueryInstalledDomainDeclarationContext, WorthQueryNeighborhoodCapableGrouping,
    WorthQuerySignalCompatiblePosture, WorthQueryTemporalDeclarationClause,
    WorthQueryTemporalDeclarationSupport, WorthQueryTemporalDuration,
};
use crate::runtime::WorthQueryRuntimeFamilySupportStatus;
use envelope::future_supported_runtime_envelope_checked;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FutureSignalDomain;

impl WorthQueryDomainEntryMarker for FutureSignalDomain {
    fn domain_key(&self) -> &'static str {
        "test.future.signal.domain"
    }

    fn display_name(&self) -> &'static str {
        "FutureSignalDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FutureSignalWorld(pub(crate) &'static str);

impl WorthQueryDomainOperatingContext<FutureSignalDomain> for FutureSignalWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[
            WorthQueryCapabilityFamily::QueryComposition,
            WorthQueryCapabilityFamily::WorkflowOrchestration,
            WorthQueryCapabilityFamily::PreviewSession,
        ]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[
            WorthQueryConfigSectionFamily::Query,
            WorthQueryConfigSectionFamily::RuntimeBridge,
            WorthQueryConfigSectionFamily::Signal,
        ]
    }

    fn required_operating_requirements(&self) -> &'static [WorthQueryDomainOperatingRequirement] {
        &[
            WorthQueryDomainOperatingRequirement::TemporalQuery,
            WorthQueryDomainOperatingRequirement::AsyncResourceQuery,
        ]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { format!("future-signal-world-{}", self.0) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

macro_rules! define_family {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) struct $name;

        impl WorthQueryDeclarationFamilyMarker<FutureSignalDomain> for $name {
            type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
            type SignalCompatibility = WorthQuerySignalCompatiblePosture;
            type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

            fn semantic_family_key() -> &'static str {
                stringify!($name)
            }

            fn legality_contract() -> WorthQueryDeclarationLegalityContract {
                WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
            }

            fn route_contract() -> WorthQueryDeclarationRouteContract {
                WorthQueryDeclarationRouteContract::bridge_only()
            }

            fn bridge_continuation_contract(
            ) -> Option<crate::application::WorthQueryDeclarationBridgeContinuationContract> {
                Some(
                    crate::application::WorthQueryDeclarationBridgeContinuationContract::runtime_route_current(),
                )
            }

            fn signal_compatibility_contract(
            ) -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
                Some(
                    WorthQueryDeclarationSignalCompatibilityContract::runtime_derived_execution(),
                )
            }

            fn aspect_contract() -> WorthQueryDeclarationAspectContract {
                WorthQueryDeclarationAspectContract::from_slices(
                    &["selection.face"],
                    &[],
                    &[],
                    &[],
                    &[],
                )
            }

            fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
                WorthQueryDeclarationAspectCoverage::from_slices(&["selection.face"], &[], &[])
            }
        }
    };
}

define_family!(OrdinaryFutureSignalFamily);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TemporalFutureSignalFamily;

impl WorthQueryDeclarationFamilyMarker<FutureSignalDomain> for TemporalFutureSignalFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "TemporalFutureSignalFamily"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }

    fn bridge_continuation_contract(
    ) -> Option<crate::application::WorthQueryDeclarationBridgeContinuationContract> {
        Some(
            crate::application::WorthQueryDeclarationBridgeContinuationContract::runtime_route_current(),
        )
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        Some(WorthQueryDeclarationSignalCompatibilityContract::runtime_derived_execution())
    }

    fn temporal_declaration_support() -> WorthQueryTemporalDeclarationSupport {
        WorthQueryTemporalDeclarationSupport::CanonicalIdentityOnly
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(&["selection.face"], &[], &[], &[], &[])
    }

    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        WorthQueryDeclarationAspectCoverage::from_slices(&["selection.face"], &[], &[])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AsyncFutureSignalFamily;

impl WorthQueryDeclarationFamilyMarker<FutureSignalDomain> for AsyncFutureSignalFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "AsyncFutureSignalFamily"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }

    fn bridge_continuation_contract(
    ) -> Option<crate::application::WorthQueryDeclarationBridgeContinuationContract> {
        Some(
            crate::application::WorthQueryDeclarationBridgeContinuationContract::runtime_route_current(),
        )
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        Some(WorthQueryDeclarationSignalCompatibilityContract::runtime_derived_execution())
    }

    fn async_declaration_support() -> WorthQueryAsyncDeclarationSupport {
        WorthQueryAsyncDeclarationSupport::CanonicalIdentityOnly
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(&["selection.face"], &[], &[], &[], &[])
    }

    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        WorthQueryDeclarationAspectCoverage::from_slices(&["selection.face"], &[], &[])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FutureSignalInput<F> {
    id: &'static str,
    _marker: PhantomData<F>,
}

impl<F> FutureSignalInput<F> {
    pub(crate) fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<FutureSignalDomain>
    for FutureSignalInput<OrdinaryFutureSignalFamily>
{
    type Family = OrdinaryFutureSignalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

impl WorthQueryDeclarationInput<FutureSignalDomain>
    for FutureSignalInput<TemporalFutureSignalFamily>
{
    type Family = TemporalFutureSignalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }

    fn temporal_declaration_clauses(&self) -> Vec<WorthQueryTemporalDeclarationClause> {
        vec![WorthQueryTemporalDeclarationClause::stale_after(
            WorthQueryTemporalDuration::seconds(30),
        )]
    }
}

impl WorthQueryDeclarationInput<FutureSignalDomain> for FutureSignalInput<AsyncFutureSignalFamily> {
    type Family = AsyncFutureSignalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }

    fn async_resource_declaration_clauses(&self) -> Vec<WorthQueryAsyncDeclarationClause> {
        vec![WorthQueryAsyncDeclarationClause::resource_request(
            WorthQueryAsyncSourceFamily::BridgeResource,
            WorthQueryAsyncLoadingPosture::Blocking,
            WorthQueryAsyncFailurePosture::FailClosed,
            vec![WorthQueryAsyncRequestIdentityPart::text("id", self.id)],
        )]
    }
}

pub(crate) fn future_signal_admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> WorthQueryInstalledDomainDeclarationContext<FutureSignalDomain, FutureSignalWorld> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        FutureSignalDomain,
        FutureSignalWorld(world),
        [
            crate::application::domain_test_support::family::<
                FutureSignalDomain,
                OrdinaryFutureSignalFamily,
            >(),
            crate::application::domain_test_support::family::<
                FutureSignalDomain,
                TemporalFutureSignalFamily,
            >(),
            crate::application::domain_test_support::family::<
                FutureSignalDomain,
                AsyncFutureSignalFamily,
            >(),
        ],
    )
}

pub(crate) fn future_signal_bridge_request() -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute,
        WorthQueryDeclarationBridgeTruthContext::Current,
    )
}

pub(crate) fn checked_signal_public_posture<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<FutureSignalDomain, FutureSignalWorld>,
    input: FutureSignalInput<F>,
) -> WorthQueryDeclarationSignalCompatibilityChecked<FutureSignalDomain, FutureSignalInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<FutureSignalDomain>,
    FutureSignalInput<F>: WorthQueryDeclarationInput<FutureSignalDomain, Family = F> + Clone,
{
    let envelope_checked = future_supported_runtime_envelope_checked(handle, input);
    let support = handle.signal_compatibility_support::<FutureSignalInput<F>>();
    worth_query_checked_declaration_signal_compatibility_on_handle(
        handle.handle_identity_digest(),
        handle.operating_context_identity_digest(),
        support.rows(),
        WorthQueryDeclarationSignalCompatibilityInput::envelope_checked(envelope_checked),
    )
}

pub(crate) fn checked_signal_supported_runtime_test_posture<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<FutureSignalDomain, FutureSignalWorld>,
    input: FutureSignalInput<F>,
) -> WorthQueryDeclarationSignalCompatibilityChecked<FutureSignalDomain, FutureSignalInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<FutureSignalDomain>,
    FutureSignalInput<F>: WorthQueryDeclarationInput<FutureSignalDomain, Family = F> + Clone,
{
    let contract = F::signal_compatibility_contract()
        .expect("future test family must expose a signal contract");
    let support_rows = contract
        .required_basis_families()
        .iter()
        .copied()
        .map(|basis_family| {
            WorthQueryDeclarationSignalCompatibilitySupportRow::new(
                contract.execution_family(),
                basis_family,
                contract.dependency_aspects(),
                contract.produced_aspects(),
                F::aspect_coverage(),
                crate::application::WorthQueryDeclarationAspectFit::Exact,
                None,
                WorthQueryDeclarationSignalCompatibilitySupportStatus::Admitted,
                "supported runtime test posture admits this future signal compatibility row",
            )
        })
        .collect::<Vec<_>>();
    let envelope_checked = future_supported_runtime_envelope_checked(handle, input);
    worth_query_checked_declaration_signal_compatibility_on_handle(
        handle.handle_identity_digest(),
        handle.operating_context_identity_digest(),
        &support_rows,
        WorthQueryDeclarationSignalCompatibilityInput::envelope_checked(envelope_checked),
    )
}
