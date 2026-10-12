use worth_foundational::facade::FoundationalBoundaryEvidenceMaterializationProfile;

use crate::application::{
    WorthQueryDeclarationEntryOrchestrationExposureLevel,
    WorthQueryDeclarationEntryOrchestrationMaterializationTier,
    WorthQueryDeclarationEntryOrchestrationProduct, WorthQueryDeclarationEntryOrchestrationStage,
    WorthQueryDeclarationEntryOrchestrationVerbCeiling,
    WorthQueryDeclarationEntryOrchestrationVerbFamily,
    WorthQueryDeclarationEntryOrchestrationVerbInventory,
};

use super::super::sequencing::WorthQueryDeclarationEntryOrchestrationAutomationParityReceipt;
use super::domain::{admitted_handle, AdmittedFamily, DeferredRouteFamily, Input};
use super::explicit_paths::{explicit_deferred_route_path_parity, explicit_success_path_parity};

#[test]
fn explicit_and_orchestrated_success_paths_produce_matching_parity_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let explicit = explicit_success_path_parity(&handle);
    let orchestrated =
        handle.orchestrate_declaration_entry_checked(Input::<AdmittedFamily>::new("edge:42"));
    let parity = WorthQueryDeclarationEntryOrchestrationAutomationParityReceipt::new(
        explicit.outcome_identity_digest().to_string(),
        orchestrated.outcome_identity_digest(),
        explicit.stop_stage(),
        orchestrated.stop_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed,
        orchestrated.stop_stage(),
    );

    assert_eq!(
        parity.explicit_outcome_identity_digest(),
        parity.orchestrated_outcome_identity_digest()
    );
    assert_eq!(
        parity.explicit_stop_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed
    );
    assert_eq!(
        parity.orchestrated_stop_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed
    );
    assert_eq!(
        parity.explicit_farthest_crossed_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed
    );
    assert_eq!(
        parity.orchestrated_farthest_crossed_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed
    );
    assert!(parity.parity_holds());
}

#[test]
fn explicit_and_orchestrated_deferred_route_paths_agree_on_receipt_stop() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let explicit = explicit_deferred_route_path_parity(&handle);
    let orchestrated =
        handle.orchestrate_declaration_entry_checked(Input::<DeferredRouteFamily>::new("edge:42"));
    let parity = WorthQueryDeclarationEntryOrchestrationAutomationParityReceipt::new(
        explicit.outcome_identity_digest().to_string(),
        orchestrated.outcome_identity_digest(),
        explicit.stop_stage(),
        orchestrated.stop_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
        WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
    );

    assert_eq!(
        parity.explicit_stop_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued
    );
    assert_eq!(
        parity.orchestrated_stop_stage(),
        WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued
    );
    assert!(parity.parity_holds());
}

#[test]
fn grammar_inventory_freezes_the_generic_trio_and_envelope_ceiling() {
    let inventory = WorthQueryDeclarationEntryOrchestrationVerbInventory::current();
    let verbs = inventory.verbs();

    assert_eq!(verbs.len(), 21);
    assert_eq!(verbs[0].public_name(), "orchestrate_declaration_entry");
    assert_eq!(
        verbs[20].public_name(),
        "orchestrate_envelope_from_progressed_proof_with_intent"
    );
    assert_eq!(
        verbs
            .iter()
            .filter(|verb| {
                verb.family()
                    == WorthQueryDeclarationEntryOrchestrationVerbFamily::GenericDeclarationEntry
            })
            .count(),
        3
    );
    assert_eq!(
        verbs
            .iter()
            .filter(|verb| {
                verb.family()
                    == WorthQueryDeclarationEntryOrchestrationVerbFamily::RouteFromProgressed
            })
            .count(),
        6
    );
    assert_eq!(
        verbs
            .iter()
            .filter(|verb| {
                verb.family()
                    == WorthQueryDeclarationEntryOrchestrationVerbFamily::ReceiptFromProgressed
            })
            .count(),
        6
    );
    assert_eq!(
        verbs
            .iter()
            .filter(|verb| {
                verb.family()
                    == WorthQueryDeclarationEntryOrchestrationVerbFamily::EnvelopeFromProgressed
            })
            .count(),
        6
    );
    assert!(verbs.iter().all(|verb| {
        verb.ceiling() == WorthQueryDeclarationEntryOrchestrationVerbCeiling::Envelope
    }));
    assert_eq!(
        verbs[..3]
            .iter()
            .map(|verb| verb.canonical_base_name())
            .collect::<Vec<_>>(),
        vec![
            "orchestrate_declaration_entry",
            "orchestrate_declaration_entry",
            "orchestrate_declaration_entry",
        ]
    );
    assert_eq!(
        verbs[..3]
            .iter()
            .map(|verb| verb.exposure_level())
            .collect::<Vec<_>>(),
        vec![
            WorthQueryDeclarationEntryOrchestrationExposureLevel::Ordinary,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::ProofVisible,
        ]
    );
    assert_eq!(
        verbs
            .iter()
            .filter(|verb| {
                verb.product() == WorthQueryDeclarationEntryOrchestrationProduct::RoutePlan
            })
            .count(),
        6
    );
    assert_eq!(
        verbs
            .iter()
            .filter(|verb| {
                verb.product() == WorthQueryDeclarationEntryOrchestrationProduct::Receipt
            })
            .count(),
        6
    );
    assert_eq!(
        verbs
            .iter()
            .filter(|verb| {
                verb.product() == WorthQueryDeclarationEntryOrchestrationProduct::Envelope
            })
            .count(),
        9
    );
}

#[test]
fn foundational_profile_mapping_stays_explicit_and_stable() {
    use crate::application::declaration_entry_orchestration::materialization::foundational_profile_for_tier;

    assert_eq!(
        foundational_profile_for_tier(
            WorthQueryDeclarationEntryOrchestrationMaterializationTier::OperationalLean
        ),
        FoundationalBoundaryEvidenceMaterializationProfile::ElideSupportAndDiagnostics
    );
    assert_eq!(
        foundational_profile_for_tier(
            WorthQueryDeclarationEntryOrchestrationMaterializationTier::SupportReady
        ),
        FoundationalBoundaryEvidenceMaterializationProfile::ElideDiagnostics
    );
    assert_eq!(
        foundational_profile_for_tier(
            WorthQueryDeclarationEntryOrchestrationMaterializationTier::FullDescriptive
        ),
        FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness
    );
}
