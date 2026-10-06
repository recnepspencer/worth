use super::*;
use crate::capabilities::DurabilityRead;

#[test]
fn worth_query_9_16_1_1_native_tail_recovers_through_current_schema_authority() {
    let runtime = persisted_runtime_with_test_schema();
    install_legacy_segment(
        &runtime,
        include_str!("../../../../tests/fixtures/worth_query_9_16_1_1_native_segment.hex"),
    );
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    assert_eq!(plan.integrity_report.corrupt_segment_id, None);
    assert_eq!(plan.tail_commit_count(), 1);
    assert_eq!(
        plan.tail_log[0].envelope().schema_authority,
        runtime.config.schema.registry.authority_snapshot()
    );
    let native_entity = legacy_created_entity(&plan.tail_log[0]);

    let mut recovered = persisted_runtime_with_test_schema();
    let outcome = recovered
        .durability_recovery()
        .recover(plan)
        .expect("current runtime recovers the legacy native tail");
    assert_eq!(outcome.recovered_commits, 1);
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("legacy main head is restored")
            .commit_id
            .0,
        1
    );
    assert_eq!(
        recovered
            .publication()
            .artifacts()
            .latest_patch()
            .expect("legacy patch is restored")
            .position,
        crate::publication::patch::data::PatchStreamPosition(1)
    );
    assert_eq!(
        observed_legacy_entity_name(&recovered, "main", native_entity),
        Some("9.16.1.1-native-fixture".to_owned())
    );

    let continued = create_entity_outcome(&recovered, "post-legacy-recovery");
    assert_eq!(
        continued.patch_position(),
        crate::publication::patch::data::PatchStreamPosition(2)
    );
    assert_eq!(
        recovered
            .durable_store()
            .expect("continued runtime retains its native store")
            .segments
            .len(),
        2,
        "a current write rolls past, rather than rewrites, the legacy segment"
    );
}

#[test]
fn worth_query_9_16_1_1_native_tail_rejects_schema_authority_substitution() {
    let runtime = persisted_runtime_with_mismatched_legacy_registry();
    install_legacy_segment(
        &runtime,
        include_str!("../../../../tests/fixtures/worth_query_9_16_1_1_native_segment.hex"),
    );
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    assert_eq!(plan.integrity_report.corrupt_segment_id, None);

    let mut recovered = persisted_runtime_with_mismatched_legacy_registry();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("a configured registry cannot replace mismatched legacy authority");
    assert_eq!(
        error.class,
        crate::durability::data::RecoveryFailureClass::SchemaMismatch
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
}

#[test]
fn worth_query_9_16_1_1_non_main_tail_reconstructs_its_parent_branch() {
    let runtime = persisted_runtime_with_test_schema();
    install_legacy_segment(
        &runtime,
        include_str!("../../../../tests/fixtures/worth_query_9_16_1_1_non_main_segment.hex"),
    );
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    assert_eq!(plan.tail_commit_count(), 2);
    let main_entity = legacy_created_entity(&plan.tail_log[0]);
    let feature_entity = legacy_created_entity(&plan.tail_log[1]);

    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(plan)
        .expect("legacy non-main branch is reconstructed from its first parent");
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("legacy-feature".to_owned()))
            .expect("legacy feature branch recovers")
            .commit_id,
        crate::history::data::CommitId(2)
    );
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("legacy parent branch remains exact")
            .commit_id,
        crate::history::data::CommitId(1)
    );
    assert_eq!(
        observed_legacy_entity_name(&recovered, "main", main_entity),
        Some("legacy-main-parent".to_owned())
    );
    assert_eq!(
        observed_legacy_entity_name(&recovered, "main", feature_entity),
        None
    );
    assert_eq!(
        observed_legacy_entity_name(&recovered, "legacy-feature", main_entity),
        Some("legacy-main-parent".to_owned())
    );
    assert_eq!(
        observed_legacy_entity_name(&recovered, "legacy-feature", feature_entity),
        Some("legacy-feature-write".to_owned())
    );
}

#[test]
fn worth_query_9_16_1_1_metadata_lineage_tail_preserves_lineage_semantics() {
    let runtime = persisted_runtime_with_test_schema();
    install_legacy_segment(
        &runtime,
        include_str!(
            "../../../../tests/fixtures/worth_query_9_16_1_1_metadata_lineage_segment.hex"
        ),
    );
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    assert_eq!(
        plan.tail_commit_count(),
        3,
        "legacy metadata fixture read error: {:?}",
        plan.persisted_terminal_error
    );
    assert_eq!(
        plan.tail_log[2].envelope().authority_kind(),
        crate::history::data::CanonicalCommitAuthorityKind::BranchReferenceMovement
    );
    let left_entity = legacy_created_entity(&plan.tail_log[0]);
    let right_entity = legacy_created_entity(&plan.tail_log[1]);
    let translated = plan.tail_log[2].envelope();
    assert!(translated.lineage_events().iter().any(|event| {
        event.event_id() == 3
            && event.kind() == LineageEventKind::Replace
            && event.sources() == [LineageId(1)]
            && event.targets() == [LineageId(2)]
    }));
    assert!(translated.lineage_decision_log().iter().any(|decision| {
        decision.kind() == &crate::lineage::data::LineageDecisionKind::ReplaceAccepted
            && decision.event_id() == Some(3)
            && decision.sources() == [LineageId(1)]
            && decision.targets() == [LineageId(2)]
    }));

    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(plan)
        .expect("legacy metadata-lineage publication recovers");
    let envelope = recovered
        .history()
        .commit_envelope(crate::history::data::CommitId(3))
        .expect("metadata-lineage envelope recovers");
    assert!(envelope
        .lineage_events()
        .iter()
        .any(|event| event.kind == LineageEventKind::Replace));
    assert!(envelope.patch.authoritative_record_patches.is_empty());
    assert_eq!(
        observed_legacy_entity_name(&recovered, "main", left_entity),
        Some("legacy-lineage-left".to_owned())
    );
    assert_eq!(
        observed_legacy_entity_name(&recovered, "main", right_entity),
        Some("legacy-lineage-right".to_owned())
    );
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("metadata-only lineage advances currentness")
            .commit_id,
        crate::history::data::CommitId(3)
    );
}

#[test]
fn worth_query_9_16_1_1_rejected_correspondence_is_typed_unsupported() {
    let runtime = persisted_runtime_with_test_schema();
    install_legacy_segment(
        &runtime,
        include_str!(
            "../../../../tests/fixtures/worth_query_9_16_1_1_rejected_lineage_segment.hex"
        ),
    );
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let tail_error = plan
        .persisted_terminal_error
        .as_ref()
        .expect("unsupported legacy lineage is explicit");
    assert_eq!(
        tail_error.class,
        crate::durability::data::RecoveryFailureClass::UnsupportedLegacySemantics
    );
    assert!(tail_error.detail.contains("correspondence rejection"));

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("unsupported legacy lineage cannot be silently discarded");
    assert_eq!(
        error.class,
        crate::durability::data::RecoveryFailureClass::UnsupportedLegacySemantics
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

fn legacy_created_entity(
    commit: &crate::durability::migration::ReadmittedCanonicalCommit,
) -> crate::identity::data::EntityId {
    match &commit.envelope().patch.authoritative_record_patches[0].target {
        crate::transactions::data::RecordRef::Entity(entity) => *entity,
        crate::transactions::data::RecordRef::Relation(_) => {
            panic!("legacy fixture creates entity")
        }
    }
}

fn observed_legacy_entity_name(
    runtime: &crate::runtime::RelationalRuntime,
    branch: &str,
    entity: crate::identity::data::EntityId,
) -> Option<String> {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("legacy branch identity recovers");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("legacy branch root is owner-admitted");
    let read = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("legacy observation transaction binds")
        .read_entity(entity)
        .expect("legacy entity read is admitted");
    read.base().and_then(read_entity_name)
}

fn install_legacy_segment(runtime: &crate::runtime::RelationalRuntime, fixture: &str) {
    let layout = runtime
        .config
        .durability
        .policy
        .store_layout
        .clone()
        .expect("persisted test runtime has a store layout");
    crate::durability::log::local_store::load_store_from_disk(runtime)
        .expect("legacy store manifest initializes");
    let segment_path = crate::durability::log::local_store::segment_file_path(
        &layout,
        crate::durability::data::DurableSegmentId(1),
    );
    std::fs::write(&segment_path, decode_hex(fixture))
        .expect("real 9.16.1.1 segment is installed in the native store");
}

fn persisted_runtime_with_mismatched_legacy_registry() -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(AspectSchemaFixture::default().build_registry())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("worth-relational-legacy-mismatch"),
            segment_commit_capacity: 2,
        })
        .build()
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    let digits = encoded
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert_eq!(digits.len() % 2, 0, "fixture hex has complete bytes");
    digits
        .chunks_exact(2)
        .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
        .collect()
}

fn hex_digit(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        b'a'..=b'f' => digit - b'a' + 10,
        _ => panic!("fixture contains a non-hex digit"),
    }
}
