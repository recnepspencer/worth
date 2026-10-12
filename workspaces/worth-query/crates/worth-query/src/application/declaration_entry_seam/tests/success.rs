use crate::application::{
    WorthQueryDeclarationBridgeRoutingSupportStatus, WorthQueryDeclarationEntryReadinessStatus,
    WorthQueryDeclarationEntrySeamClassification,
    WorthQueryDeclarationRelationalTruthRoutingSupportStatus,
    WorthQueryDeclarationSignalCompatibilitySupportStatus,
};

use super::support::{
    handle, signal_disabled_handle, BridgeSignalFamily, DeferredSignalFamily, Input, MixedFamily,
    RelationalFamily,
};

#[test]
fn inventory_digest_changes_when_admitted_world_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = handle(request_execution, "alpha")
        .declaration_entry_crossing_inventory::<Input<RelationalFamily>>();
    let right = handle(request_execution, "beta")
        .declaration_entry_crossing_inventory::<Input<RelationalFamily>>();

    assert_ne!(left.inventory_digest(), right.inventory_digest());
}

#[test]
fn mixed_family_inventory_keeps_relational_and_bridge_rows_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let inventory = handle(request_execution, "mixed")
        .declaration_entry_crossing_inventory::<Input<MixedFamily>>();
    assert!(inventory
        .rows()
        .iter()
        .any(|row| row.relational_truth_claim().is_some()));
    assert!(inventory
        .rows()
        .iter()
        .any(|row| row.bridge_continuation_family().is_some()));
    assert!(inventory
        .rows()
        .iter()
        .any(|row| row.entrypoint_key() == "route-relational-truth-checked"));
    assert!(inventory
        .rows()
        .iter()
        .any(|row| row.entrypoint_key() == "route-bridge-continuation-from-progressed"));
}

#[test]
fn deferred_signal_family_uses_deferred_neighbor_classification() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let inventory = handle(request_execution, "deferred")
        .declaration_entry_crossing_inventory::<Input<DeferredSignalFamily>>();
    let signal_row = inventory
        .rows()
        .iter()
        .find(|row| {
            matches!(
                row.surface(),
                crate::application::WorthQueryDeclarationEntryCrossingSurface::SignalCompatibility
            )
        })
        .expect("deferred signal row should exist");
    assert_eq!(
        signal_row.seam_classification(),
        WorthQueryDeclarationEntrySeamClassification::DeferredNeighbor
    );
}

#[test]
fn readiness_projection_matches_signal_support_status() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let readiness = handle.declaration_entry_readiness::<Input<BridgeSignalFamily>>();
    let signal_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().signal_execution_family().is_some())
        .expect("signal row should exist");
    assert_eq!(
        signal_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );

    let signal_support = handle.signal_compatibility_support::<Input<BridgeSignalFamily>>();
    assert_eq!(signal_support.rows().len(), 1);
    assert_eq!(
        signal_support.rows()[0].status(),
        WorthQueryDeclarationSignalCompatibilitySupportStatus::Admitted
    );
    assert_eq!(signal_support.rows()[0].reason(), signal_row.reason());
}

#[test]
fn mixed_family_readiness_matches_relational_and_bridge_support_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "mixed");
    let readiness = handle.declaration_entry_readiness::<Input<MixedFamily>>();
    let relational_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().relational_truth_claim().is_some())
        .expect("relational readiness row should exist");
    let bridge_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge readiness row should exist");

    let relational_support = handle.relational_truth_support::<Input<MixedFamily>>();
    assert_eq!(relational_support.rows().len(), 1);
    assert_eq!(
        relational_support.rows()[0].status(),
        WorthQueryDeclarationRelationalTruthRoutingSupportStatus::Admitted
    );
    assert_eq!(
        relational_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );
    assert_eq!(
        relational_support.rows()[0].reason(),
        relational_row.reason()
    );

    let bridge_support = handle.bridge_continuation_support::<Input<MixedFamily>>();
    assert_eq!(bridge_support.rows().len(), 1);
    assert_eq!(
        bridge_support.rows()[0].status(),
        WorthQueryDeclarationBridgeRoutingSupportStatus::Admitted
    );
    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );
    assert_eq!(bridge_support.rows()[0].reason(), bridge_row.reason());
}

#[test]
fn signalless_world_keeps_invalid_basis_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = signal_disabled_handle(request_execution, "preview");
    let readiness = handle.declaration_entry_readiness::<Input<BridgeSignalFamily>>();
    let signal_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().signal_execution_family().is_some())
        .expect("signal row should exist");
    assert_eq!(
        signal_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::InvalidBasis
    );

    let signal_support = handle.signal_compatibility_support::<Input<BridgeSignalFamily>>();
    assert_eq!(signal_support.rows().len(), 1);
    assert_eq!(
        signal_support.rows()[0].status(),
        WorthQueryDeclarationSignalCompatibilitySupportStatus::InvalidBasis
    );
    assert_eq!(signal_support.rows()[0].reason(), signal_row.reason());
}
