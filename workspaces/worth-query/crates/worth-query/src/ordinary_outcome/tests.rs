use std::marker::PhantomData;

use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationEntryOrchestrationAutomationBoundary,
    WorthQueryDeclarationEntryOrchestrationAutomationRefusal,
    WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass,
    WorthQueryDeclarationEntryOrchestrationDenied, WorthQueryDeclarationEntryOrchestrationRefusal,
    WorthQueryDeclarationEntryOrchestrationStage, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationRouteContract, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryNeighborhoodCapableGrouping,
    WorthQueryRelationalTruthAuthority, WorthQuerySignalCompatiblePosture,
};
use crate::binding_pipeline::{
    WorthQueryBindingChecked, WorthQueryBindingLinkedArtifacts, WorthQueryDeclarationBindingRequest,
};
use crate::facade::foundation::{
    WorthQueryBindingOutcome, WorthQueryBindingSourceKind, WorthQueryBindingWrongWorld,
    WorthQueryDeclarationEntryOrchestrationRefusalClass,
    WorthQueryDeclarationEntryOrchestrationTerminalError,
};
use crate::ordinary_outcome::{
    ordinary_outcome_from_binding_outcome, ordinary_outcome_from_orchestration_terminal,
    WorthQueryOrdinaryBindingCheckedTopologyKind, WorthQueryOrdinaryNextStep,
    WorthQueryOrdinaryOutcome, WorthQueryOrdinaryPostureKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OrdinaryDomain;

impl WorthQueryDomainEntryMarker for OrdinaryDomain {
    fn domain_key(&self) -> &'static str {
        "test.ordinary.domain"
    }

    fn display_name(&self) -> &'static str {
        "OrdinaryDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OrdinaryWorld;

impl WorthQueryDomainOperatingContext<OrdinaryDomain> for OrdinaryWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::HistoricalEvaluation]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[
            WorthQueryConfigSectionFamily::Query,
            WorthQueryConfigSectionFamily::Relational,
            WorthQueryConfigSectionFamily::RuntimeBridge,
        ]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { "ordinary-world".to_string() };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OrdinaryFamily;

impl WorthQueryDeclarationFamilyMarker<OrdinaryDomain> for OrdinaryFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "OrdinaryFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(&["selection.edge"], &[], &[], &[], &[])
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::relational_only()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExpensiveFamily;

impl WorthQueryDeclarationFamilyMarker<OrdinaryDomain> for ExpensiveFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "ExpensiveFamily"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::expensive_by_default_for_tests()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Input<F> {
    id: &'static str,
    _marker: PhantomData<F>,
}

impl<F> Input<F> {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<OrdinaryDomain> for Input<OrdinaryFamily> {
    type Family = OrdinaryFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

impl WorthQueryDeclarationInput<OrdinaryDomain> for Input<ExpensiveFamily> {
    type Family = ExpensiveFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

#[test]
fn ordinary_binding_projection_keeps_wrong_world_distinct() {
    let checked = WorthQueryBindingChecked::new(
        WorthQueryBindingOutcome::<()>::WrongWorld(WorthQueryBindingWrongWorld::new("wrong world")),
        "digest".to_string(),
        WorthQueryBindingLinkedArtifacts::new(),
    );
    let ordinary = ordinary_outcome_from_binding_outcome(checked);
    match ordinary {
        WorthQueryOrdinaryOutcome::WrongWorld(posture) => {
            assert_eq!(posture.kind(), WorthQueryOrdinaryPostureKind::WrongWorld);
            assert_eq!(
                posture.next_step(),
                WorthQueryOrdinaryNextStep::CorrectWorld
            );
            assert_eq!(
                posture.checked_topology().binding_kind(),
                Some(WorthQueryOrdinaryBindingCheckedTopologyKind::WrongWorld)
            );
            assert!(posture
                .checked_topology()
                .binding_linked_artifacts()
                .is_some());
        }
        other => panic!(
            "unexpected ordinary outcome: {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn ordinary_orchestration_projection_preserves_route_stop_stage() {
    let terminal = WorthQueryDeclarationEntryOrchestrationTerminalError::<
        OrdinaryDomain,
        Input<OrdinaryFamily>,
    >::Denied(WorthQueryDeclarationEntryOrchestrationDenied::new(
        OrdinaryFamily::semantic_family_key(),
        WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
        "route denied",
        Some("route-denial".to_string()),
    ));
    let ordinary = ordinary_outcome_from_orchestration_terminal(terminal);
    match ordinary {
        WorthQueryOrdinaryOutcome::Denied(posture) => {
            assert_eq!(posture.kind(), WorthQueryOrdinaryPostureKind::Denied);
            assert_eq!(
                posture.checked_topology().orchestration_stop_stage(),
                Some(WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned)
            );
        }
        other => panic!(
            "unexpected ordinary outcome: {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn ordinary_orchestration_projection_preserves_refusal_class() {
    let automation = WorthQueryDeclarationEntryOrchestrationAutomationRefusal::new(
        WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::ExpensiveAutomationForbidden,
        WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
        "explicit handoff required",
        ExpensiveFamily::semantic_family_key(),
        Some("expensive-route".to_string()),
        "orchestration-digest",
        WorthQueryDeclarationEntryOrchestrationAutomationBoundary::EnvelopeCeiling,
    );
    let terminal = WorthQueryDeclarationEntryOrchestrationTerminalError::<
        OrdinaryDomain,
        Input<ExpensiveFamily>,
    >::Refused(
        WorthQueryDeclarationEntryOrchestrationRefusal::from_automation(
            automation,
            WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
        ),
    );
    let ordinary = ordinary_outcome_from_orchestration_terminal(terminal);
    match ordinary {
        WorthQueryOrdinaryOutcome::Refused(posture) => {
            assert_eq!(posture.kind(), WorthQueryOrdinaryPostureKind::Refused);
            assert_eq!(
                posture.next_step(),
                WorthQueryOrdinaryNextStep::UseExplicitHandoff
            );
            assert_eq!(
                posture.checked_topology().orchestration_refusal_class(),
                Some(
                    WorthQueryDeclarationEntryOrchestrationRefusalClass::ExpensiveWorkNotAdmittedByDefault
                )
            );
        }
        other => panic!(
            "unexpected ordinary outcome: {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn admitted_handle_exposes_ordinary_binding_and_orchestration_entrypoints() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        OrdinaryDomain,
        OrdinaryWorld,
        [
            crate::application::domain_test_support::family::<OrdinaryDomain, OrdinaryFamily>(),
            crate::application::domain_test_support::family::<OrdinaryDomain, ExpensiveFamily>(),
        ],
    );

    let bind_outcome =
        handle.bind_declaration_from_context_outcome(WorthQueryDeclarationBindingRequest::<
            Input<OrdinaryFamily>,
        >::new(
            vec![],
            OrdinaryFamily::aspect_contract(),
            vec![WorthQueryBindingSourceKind::ExplicitSelection],
        ));
    assert!(matches!(
        bind_outcome,
        WorthQueryOrdinaryOutcome::Unavailable(_)
    ));

    let orchestration_outcome =
        handle.orchestrate_declaration_entry_outcome(Input::<ExpensiveFamily>::new("edge-a"));
    assert!(matches!(
        orchestration_outcome,
        WorthQueryOrdinaryOutcome::Refused(_)
    ));
}
