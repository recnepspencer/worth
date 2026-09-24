use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

use crate::durability::data::RecoveryFailureClass;
use crate::facade::durability::RecoveryVerificationMode;
use crate::facade::history::BranchId;
use crate::facade::identity::{EntityId, KindId, PartitionId};
use crate::facade::mvcc::WorkerIntentBatch;
use crate::facade::runtime::{RelationalFieldPresence, RelationalFieldRevision};
use crate::facade::snapshots::SnapshotHandle;
use crate::facade::transactions::{
    ApplyEntityAspectPatchIntent, CreateIntent, EntityMutationIntent, MutationIntent,
    UpdateEntityFieldsIntent,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
};

fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key(aspect),
        CanonicalFieldPath::single(field_key(field)),
    )
}

#[test]
fn checkpoint_restore_preserves_exact_field_revision() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "same");
    let entity = changed_entities(&created)[0];
    let name = locator("name", "name");
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_field_revision(entity, &name)
        .expect("created field has a native owner revision");
    runtime.durability_authority().checkpoint().unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let mut recovered = runtime_with_test_schema();
    let outcome = recovered.durability_recovery().recover(recovery).unwrap();
    assert_eq!(outcome.coverage.checkpoint_commits, 1);
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".to_owned()));
    assert_eq!(
        recovered
            .read_truth()
            .project_snapshot(&snapshot)
            .unwrap()
            .entity_field_revision(entity, &name),
        Some(before)
    );
    recovered.snapshots().release_snapshot(&snapshot).unwrap();
    recovered.durability_authority().checkpoint().unwrap();
    let second_recovery = recovered
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let mut reopened = runtime_with_test_schema();
    reopened
        .durability_recovery()
        .recover(second_recovery)
        .unwrap();
    let reopened_snapshot = snapshot_for_owner_branch(&reopened, &BranchId("main".to_owned()));
    assert_eq!(
        reopened
            .read_truth()
            .project_snapshot(&reopened_snapshot)
            .unwrap()
            .entity_field_revision(entity, &name),
        Some(before)
    );
    reopened
        .snapshots()
        .release_snapshot(&reopened_snapshot)
        .unwrap();
    release_test_commit_snapshot(&runtime, &created);
}

#[test]
fn checkpoint_restore_preserves_relation_field_revision() {
    let fixture = || {
        AspectSchemaFixture {
            relation_aspects: vec![relation_field_aspect(
                aspect_key("label"),
                field_key("label"),
            )],
            ..AspectSchemaFixture::default()
        }
        .build_runtime()
    };
    let runtime = fixture();
    let source = create_entity_outcome(&runtime, "source");
    let target = create_entity_outcome(&runtime, "target");
    let created = create_relation_outcome(
        &runtime,
        changed_entities(&source)[0],
        changed_entities(&target)[0],
        "relation-label",
    );
    let relation = changed_relations(&created)[0];
    let label = locator("label", "label");
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .relation_field_revision(relation, &label)
        .unwrap();
    assert_eq!(before.presence(), RelationalFieldPresence::Present);

    runtime.durability_authority().checkpoint().unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let mut recovered = fixture();
    recovered.durability_recovery().recover(recovery).unwrap();
    let snapshot = snapshot_for_owner_branch(&recovered, &BranchId("main".to_owned()));
    assert_eq!(
        recovered
            .read_truth()
            .project_snapshot(&snapshot)
            .unwrap()
            .relation_field_revision(relation, &label),
        Some(before)
    );
    recovered.snapshots().release_snapshot(&snapshot).unwrap();
    for outcome in [&source, &target, &created] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn checkpoint_restore_preserves_forked_field_revisions_and_absent_aba() {
    let binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let contract = binding.contract.clone();
    let fixture = || {
        AspectSchemaFixture {
            entity_aspects: vec![binding.clone()],
            ..AspectSchemaFixture::default()
        }
        .build_runtime()
    };
    let runtime = fixture();
    let mut create = test_owner_begin_transaction_for_main(&runtime);
    create
        .push_batch(
            WorkerIntentBatch::new("checkpoint-field-create").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("field-checkpoint"),
                    fields: string_aspect_field_patch([(
                        aspect_key("summary"),
                        field_key("title"),
                        "fixed",
                    )]),
                }),
            )),
        )
        .unwrap();
    let created = create.commit(&runtime).unwrap();
    let entity = changed_entities(&created)[0];
    let prior_branch = create_branch_from_main(&runtime, "field-prior");
    let title = locator("summary", "title");
    let status = locator("summary", "status");
    let initial_title = field_at(&runtime, &created.snapshot, entity, &title).unwrap();
    let initial_absent = field_at(&runtime, &created.snapshot, entity, &status).unwrap();
    assert_eq!(initial_absent.presence(), RelationalFieldPresence::Absent);

    let mut set = test_owner_begin_transaction_for_main(&runtime);
    set.push_batch(
        WorkerIntentBatch::new("checkpoint-field-set").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: string_aspect_field_patch([(
                    aspect_key("summary"),
                    field_key("status"),
                    "temporary",
                )]),
            }),
        )),
    )
    .unwrap();
    let present = set.commit(&runtime).unwrap();
    let present_revision = field_at(&runtime, &present.snapshot, entity, &status).unwrap();
    assert_eq!(
        present_revision.presence(),
        RelationalFieldPresence::Present
    );
    let present_branch = create_branch_from_main(&runtime, "field-present");

    let mut clear = test_owner_begin_transaction_for_main(&runtime);
    clear
        .push_batch(
            WorkerIntentBatch::new("checkpoint-field-clear").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: PortableRecordAspectPatch::new([
                        PortableAspectPatchOperation::PatchFields {
                            basis: PortableAspectContractBasis::from_contract(&contract),
                            selected_fields: vec![field_key("status")],
                            field_sets: Vec::new(),
                            field_clears: vec![field_key("status")],
                        },
                    ]),
                }),
            )),
        )
        .unwrap();
    let absent_again = clear.commit(&runtime).unwrap();
    let final_absent = field_at(&runtime, &absent_again.snapshot, entity, &status).unwrap();
    assert_eq!(final_absent.presence(), RelationalFieldPresence::Absent);
    assert_ne!(initial_absent, final_absent);
    assert_eq!(
        field_at(&runtime, &absent_again.snapshot, entity, &title),
        Some(initial_title)
    );

    runtime.durability_authority().checkpoint().unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let mut recovered = fixture();
    recovered.durability_recovery().recover(recovery).unwrap();
    for (branch, expected) in [
        (prior_branch, initial_absent),
        (present_branch, present_revision),
        (BranchId("main".to_owned()), final_absent),
    ] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            field_at(&recovered, &snapshot, entity, &status),
            Some(expected)
        );
        assert_eq!(
            field_at(&recovered, &snapshot, entity, &title),
            Some(initial_title)
        );
        recovered.snapshots().release_snapshot(&snapshot).unwrap();
    }
    for outcome in [&created, &present, &absent_again] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn checkpoint_restore_rejects_incomplete_field_revision_column() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "same");
    runtime.durability_authority().checkpoint().unwrap();
    let mut recovery = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let root = &mut recovery.checkpoint.as_mut().unwrap().branch_roots[0];
    let image = &mut root.partition_images[0].entity_arena;
    assert!(!image.field_revisions.is_empty());
    image.field_revisions.pop();
    root.partition_image_digest =
        crate::durability::data::branch_root_partition_image_digest(&root.partition_images)
            .unwrap();
    root.root_image_digest = crate::durability::data::branch_root_image_digest(
        root.format_version,
        root.commit_id,
        root.partition_image_digest,
        root.schema_carrier_digest,
    );

    let mut recovered = runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(recovery)
        .unwrap_err();
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("field revision length"), "{error:?}");
    release_test_commit_snapshot(&runtime, &created);
}

fn field_at(
    runtime: &crate::runtime::RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    locator: &AspectFieldLocator,
) -> Option<RelationalFieldRevision> {
    runtime
        .read_truth()
        .project_snapshot(snapshot)?
        .entity_field_revision(entity, locator)
}
