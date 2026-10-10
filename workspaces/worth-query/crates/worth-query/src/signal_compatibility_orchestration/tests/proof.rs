use std::marker::PhantomData;

use crate::application::{
    WorthQueryBridgeContinuationAuthority, WorthQueryCapabilityFamily,
    WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationRouteContract, WorthQueryDeclarationSignalCompatibilityContract,
    WorthQueryDeclarationSignalCompatibilityInput, WorthQueryDomainOperatingContext,
    WorthQueryNeighborhoodCapableGrouping, WorthQuerySignalDeferredPosture,
    WorthQuerySignalNotCompatiblePosture,
};
use crate::ordinary_outcome::{
    WorthQueryOrdinaryOutcome,
    WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind,
};

use super::support::{orchestration_input, SignalDomain, SignalFamily, SignalWorld};

#[cfg(test)]
mod outcome_tests;

fn proof_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<SignalDomain, SignalWorld> {
    proof_signal_context(request_execution, SignalWorld(world))
}

fn proof_signal_context<C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: C,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<SignalDomain, C>
where
    C: WorthQueryDomainOperatingContext<SignalDomain>,
{
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        SignalDomain,
        context,
        [
            crate::application::domain_test_support::family::<SignalDomain, SignalFamily>(),
            crate::application::domain_test_support::family::<SignalDomain, DeferredSignalFamily>(),
            crate::application::domain_test_support::family::<
                SignalDomain,
                HistoricalBasisSignalFamily,
            >(),
            crate::application::domain_test_support::family::<SignalDomain, UnsupportedSignalFamily>(
            ),
        ],
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BasisMismatchWorld;

impl WorthQueryDomainOperatingContext<SignalDomain> for BasisMismatchWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
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
        let value = { "signal-world-basis-mismatch".to_string() };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SharedIdentityDifferentHandleWorld;

impl WorthQueryDomainOperatingContext<SignalDomain> for SharedIdentityDifferentHandleWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[
            WorthQueryCapabilityFamily::HistoricalEvaluation,
            WorthQueryCapabilityFamily::WorkflowOrchestration,
            WorthQueryCapabilityFamily::PreviewSession,
            WorthQueryCapabilityFamily::QueryComposition,
            WorthQueryCapabilityFamily::QueryRead,
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
        SignalWorld("shared").context_identity()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DeferredSignalFamily;

impl WorthQueryDeclarationFamilyMarker<SignalDomain> for DeferredSignalFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalDeferredPosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "DeferredSignalFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        SignalFamilyShape::aspect_contract()
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UnsupportedSignalFamily;

impl WorthQueryDeclarationFamilyMarker<SignalDomain> for UnsupportedSignalFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalNotCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "UnsupportedSignalFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        SignalFamilyShape::aspect_contract()
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeferredSignalInput {
    id: &'static str,
    _marker: PhantomData<DeferredSignalFamily>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UnsupportedSignalInput {
    id: &'static str,
    _marker: PhantomData<UnsupportedSignalFamily>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HistoricalBasisSignalFamily;

impl WorthQueryDeclarationFamilyMarker<SignalDomain> for HistoricalBasisSignalFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = crate::application::WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "HistoricalBasisSignalFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        SignalFamilyShape::aspect_contract()
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        Some(WorthQueryDeclarationSignalCompatibilityContract::historical_derived_execution())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HistoricalBasisSignalInput {
    id: &'static str,
    _marker: PhantomData<HistoricalBasisSignalFamily>,
}

impl DeferredSignalInput {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl UnsupportedSignalInput {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl HistoricalBasisSignalInput {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<SignalDomain> for DeferredSignalInput {
    type Family = DeferredSignalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

impl WorthQueryDeclarationInput<SignalDomain> for UnsupportedSignalInput {
    type Family = UnsupportedSignalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

impl WorthQueryDeclarationInput<SignalDomain> for HistoricalBasisSignalInput {
    type Family = HistoricalBasisSignalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

fn admitted_basis_mismatch_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<SignalDomain, BasisMismatchWorld>
{
    proof_signal_context(request_execution, BasisMismatchWorld)
}

fn admitted_same_world_different_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    SignalDomain,
    SharedIdentityDifferentHandleWorld,
> {
    proof_signal_context(request_execution, SharedIdentityDifferentHandleWorld)
}

fn local_input<I>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        SignalDomain,
        SignalWorld,
    >,
    id: &'static str,
) -> crate::signal_compatibility_orchestration::WorthQuerySignalCompatibilityOrchestrationInput<
    SignalDomain,
    I,
>
where
    I: WorthQueryDeclarationInput<SignalDomain> + LocalSignalInput,
{
    let envelope = handle
        .declare_review_progress_describe_plan_receipt_and_envelope(I::new_local(id))
        .unwrap_or_else(|_| panic!("expected envelope-backed local signal input"));
    crate::signal_compatibility_orchestration::WorthQuerySignalCompatibilityOrchestrationInput::new(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope),
    )
}

trait LocalSignalInput: Sized {
    fn new_local(id: &'static str) -> Self;
}

impl LocalSignalInput for DeferredSignalInput {
    fn new_local(id: &'static str) -> Self {
        Self::new(id)
    }
}

impl LocalSignalInput for UnsupportedSignalInput {
    fn new_local(id: &'static str) -> Self {
        Self::new(id)
    }
}

fn basis_mismatch_input(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        SignalDomain,
        BasisMismatchWorld,
    >,
    id: &'static str,
) -> crate::signal_compatibility_orchestration::WorthQuerySignalCompatibilityOrchestrationInput<
    SignalDomain,
    HistoricalBasisSignalInput,
> {
    let envelope = handle
        .declare_review_progress_describe_plan_receipt_and_envelope(
            HistoricalBasisSignalInput::new(id),
        )
        .unwrap_or_else(|_| panic!("expected envelope-backed basis-mismatch input"));
    crate::signal_compatibility_orchestration::WorthQuerySignalCompatibilityOrchestrationInput::new(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope),
    )
}

struct SignalFamilyShape;

impl SignalFamilyShape {
    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_face", "signal.dependency.runtime_inputs"],
            &["selection.neighborhood.local_topology"],
            &["signal.preview.surface"],
            &["signal.private_authority"],
            &["signal.conflicting_dependency"],
        )
    }
}
