use {
    worth_store_io_scheduler::execute_background_pressure_for_certification_test,
    worth_store_io_scheduler::mismatched_background_pressure_denial_for_certification_test,
};

// store-proof-identity[producer_pressure_declarations_lower_into_scheduler_shapes]: worth-store-io-scheduler::src/background_pacing/tests/producer_declarations/producer_pressure_declarations_lower_into_scheduler_shapes::producer_declarations::producer_pressure_declarations_lower_into_scheduler_shapes
#[test]
fn producer_pressure_declarations_lower_into_scheduler_shapes() {
    let cases = [
        (
            worth_store_physical_isolation::compaction_rewrite_scheduler_demand(),
            worth_store_io_scheduler::BackgroundIoPressureClass::CompactionRewrite,
        ),
        (
            worth_store_physical_isolation::checkpoint_flush_scheduler_demand(),
            worth_store_io_scheduler::BackgroundIoPressureClass::CheckpointFlush,
        ),
        (
            worth_store_operations::replication_prep_background_pressure_shape(4),
            worth_store_io_scheduler::BackgroundIoPressureClass::ReplicationPrepRead,
        ),
        (
            worth_store_blob_chunks::blob_ingest_background_pressure_shape(4096),
            worth_store_io_scheduler::BackgroundIoPressureClass::IngestPressure,
        ),
        (
            worth_store_blob_chunks::blob_migration_background_pressure_shape(4096),
            worth_store_io_scheduler::BackgroundIoPressureClass::MigrationPressure,
        ),
        (
            worth_store_operations::backup_prep_background_pressure_shape(4096, 4),
            worth_store_io_scheduler::BackgroundIoPressureClass::BackupPrepRead,
        ),
        (
            worth_store_operations::repair_background_pressure_shape(4),
            worth_store_io_scheduler::BackgroundIoPressureClass::RepairScan,
        ),
        (
            worth_store_offline_verifier::offline_repair_scan_background_pressure_shape(4),
            worth_store_io_scheduler::BackgroundIoPressureClass::RepairScan,
        ),
        (
            worth_store_offline_verifier::offline_verification_pressure_background_pressure_shape(
                4,
            ),
            worth_store_io_scheduler::BackgroundIoPressureClass::VerificationPressure,
        ),
    ];

    for (declaration, expected) in cases {
        assert_eq!(
            worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(declaration).class(),
            expected
        );
    }
}

// store-proof-identity[physical_isolation_declarations_lower_concrete_resource_units]: worth-store-io-scheduler::src/background_pacing/tests/producer_declarations/physical_isolation_declarations_lower_concrete_resource_units::producer_declarations::physical_isolation_declarations_lower_concrete_resource_units
#[test]
fn physical_isolation_declarations_lower_concrete_resource_units() {
    let compaction =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_physical_isolation::compaction_rewrite_scheduler_demand(),
        );
    assert!(compaction.requested_budget().queue_slots() > 0);
    assert!(compaction.requested_budget().bandwidth_tokens() > 0);
    assert!(compaction.requested_budget().write_back_window() > 0);
    assert!(compaction.requested_budget().dirty_page_budget() > 0);

    let checkpoint =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_physical_isolation::checkpoint_flush_scheduler_demand(),
        );
    assert!(checkpoint.requested_budget().flush_permits() > 0);
    assert!(checkpoint.requested_budget().sync_debt() > 0);
    assert!(checkpoint.requested_budget().write_back_window() > 0);
}

// store-proof-identity[quantity_bearing_producer_declarations_survive_scheduler_lowering]: worth-store-io-scheduler::src/background_pacing/tests/producer_declarations/quantity_bearing_producer_declarations_survive_scheduler_lowering::producer_declarations::quantity_bearing_producer_declarations_survive_scheduler_lowering
#[test]
fn quantity_bearing_producer_declarations_survive_scheduler_lowering() {
    let replication =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_operations::replication_prep_background_pressure_shape(4),
        );
    assert_eq!(replication.requested_budget().read_ahead_window(), 4);
    assert!(replication.requested_budget().queue_slots() > 0);

    let blob =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_blob_chunks::blob_ingest_background_pressure_shape(4096),
        );
    assert_eq!(blob.requested_budget().bandwidth_tokens(), 4096);
    assert!(blob.requested_budget().worker_permits() > 0);

    let backup =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_operations::backup_prep_background_pressure_shape(4096, 4),
        );
    assert_eq!(backup.requested_budget().bandwidth_tokens(), 4096);
    assert_eq!(backup.requested_budget().read_ahead_window(), 4);

    let verification =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_offline_verifier::offline_verification_pressure_background_pressure_shape(
                4,
            ),
        );
    assert_eq!(verification.requested_budget().read_ahead_window(), 4);
    assert!(verification.requested_budget().queue_slots() > 0);
}

// store-proof-identity[producer_pressure_declarations_are_scheduler_consumable]: worth-store-io-scheduler::src/background_pacing/tests/producer_declarations/producer_pressure_declarations_are_scheduler_consumable::producer_declarations::producer_pressure_declarations_are_scheduler_consumable
#[test]
fn producer_pressure_declarations_are_scheduler_consumable() {
    let cases = [
        worth_store_physical_isolation::compaction_rewrite_scheduler_demand(),
        worth_store_physical_isolation::checkpoint_flush_scheduler_demand(),
        worth_store_operations::replication_prep_background_pressure_shape(4),
        worth_store_blob_chunks::blob_migration_background_pressure_shape(4096),
        worth_store_operations::backup_prep_background_pressure_shape(4096, 4),
        worth_store_operations::repair_background_pressure_shape(4),
        worth_store_offline_verifier::offline_repair_scan_background_pressure_shape(4),
        worth_store_offline_verifier::offline_verification_pressure_background_pressure_shape(4),
    ];

    for declaration in cases {
        let shape = worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(declaration);
        let outcome = execute_background_pressure_for_certification_test(shape);
        assert!(matches!(
            outcome,
            worth_store_io_scheduler::BackgroundPacingOutcome::AdmittedWithDebt(_)
        ));
    }

    let blob_ingest =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_blob_chunks::blob_ingest_background_pressure_shape(4096),
        );
    assert!(matches!(
        mismatched_background_pressure_denial_for_certification_test(blob_ingest),
        worth_store_io_scheduler::BackgroundPacingDenial::BackendRequirementMismatch { .. }
    ));
}

// store-proof-identity[equivalent_repair_declarations_have_parity_under_same_basis]: worth-store-io-scheduler::src/background_pacing/tests/producer_declarations/equivalent_repair_declarations_have_parity_under_same_basis::producer_declarations::equivalent_repair_declarations_have_parity_under_same_basis
#[test]
fn equivalent_repair_declarations_have_parity_under_same_basis() {
    let operations =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_operations::repair_background_pressure_shape(4),
        );
    let offline =
        worth_store_io_scheduler::BackgroundIoPressureShape::from_background_pressure_declaration(
            worth_store_offline_verifier::offline_repair_scan_background_pressure_shape(4),
        );
    assert_eq!(operations.requested_budget(), offline.requested_budget());

    let operations_outcome = execute_background_pressure_for_certification_test(operations);
    let offline_outcome = execute_background_pressure_for_certification_test(offline);
    assert_eq!(operations_outcome, offline_outcome);
}
