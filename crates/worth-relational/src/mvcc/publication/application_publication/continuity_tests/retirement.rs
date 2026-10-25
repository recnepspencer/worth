//! Complete publication position survives total membership replacement.
use super::*;
#[path = "retirement/ordinary.rs"]
mod ordinary;

fn creates(keys: [&str; 2]) -> (WorkerIntentBatch, [CreatedEntityRef; 2]) {
    let targets = keys.map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let mut batch = WorkerIntentBatch::new("real-new-complete-membership");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: name_field_patch("actual payload"),
        })));
    }
    (batch, targets)
}

fn completion(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    previous: RelationalRequiredOutputPublication,
    roles: Vec<RequiredOutputRoleTarget>,
    dependency: u8,
) -> RequiredOutputPublication {
    let meaning = previous
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .meaning
        .clone();
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("retirement-preserver", [dependency; 32]),
            previous.authority(),
        )
        .unwrap();
    RequiredOutputPublication::Preserve {
        admission,
        predecessor: previous,
        meaning,
        roles,
    }
}

fn new_roles(targets: &[CreatedEntityRef; 2]) -> Vec<RequiredOutputRoleTarget> {
    targets
        .iter()
        .enumerate()
        .map(|(index, target)| RequiredOutputRoleTarget {
            role: format!("created-{index}"),
            entity_name: "native-unit-output".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Create,
            target: EntityReference::Created(target.clone()),
        })
        .collect()
}

#[test]
fn all_retired_then_disjoint_create_rejects_stale_complete_predecessor_before_movement() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let (batch, targets) = creates(["old-left", "old-right"]);
    let birth = publish(
        &runtime,
        execution,
        &installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("retirement-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| birth.created_entity(&target).unwrap());
    let original = birth.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &birth);
    let mut batch = WorkerIntentBatch::new("actual-total-retirement");
    for id in ids {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: id },
        )));
    }
    let retired_roles = ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| RequiredOutputRoleTarget {
            role: format!("retired-{index}"),
            entity_name: "native-unit-output".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Retire,
            target: EntityReference::Existing(id),
        })
        .collect::<Vec<_>>();
    let retired = publish(
        &runtime,
        execution,
        &installed,
        batch,
        completion(&owner, original, retired_roles.clone(), 2),
    )
    .unwrap();
    let previous = retired.required_output_publication(&installed).unwrap();
    let stale = previous.clone();
    let (batch, targets) = creates(["new-left", "new-right"]);
    let replacement = publish(
        &runtime,
        execution,
        &installed,
        batch,
        completion(&owner, previous, new_roles(&targets), 3),
    )
    .unwrap();
    let replacement_ids = targets.map(|target| replacement.created_entity(&target).unwrap());
    let current = replacement.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &replacement);
    let before = test_owner_main_basis(&runtime).unwrap();
    for id in ids {
        assert!(!before
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .entity_arena
            .get(&id)
            .expect("the actual old tombstone generation is retained")
            .is_live());
    }
    cold_replacement(execution, &runtime, retired_roles, replacement_ids);
    release_test_commit_snapshot(&runtime, &retired);
    let (batch, targets) = creates(["fork-left", "fork-right"]);
    let denied = publish(
        &runtime,
        execution,
        &installed,
        batch,
        completion(&owner, stale, new_roles(&targets), 4),
    )
    .unwrap_err();
    assert!(
        format!("{denied:?}").contains("not the publication selected at the native before image")
    );
    let after = test_owner_main_basis(&runtime).unwrap();
    assert_eq!(
        before.descriptor(),
        after.descriptor(),
        "no canonical branch movement or published rows"
    );
    let settled = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("actual-latest-no-data"),
        preserve(&owner, current, replacement_ids, 5),
    )
    .unwrap();
    assert!(settled.changed_records.is_empty());
    release_test_commit_snapshot(&runtime, &settled);
    let basis = test_owner_main_basis(&runtime).unwrap();
    runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-preserver",
            &[5; 32],
            &crate::mvcc::native_meaning_fixture("retirement-creator"),
            &crate::mvcc::native_roles_fixture(
                &replacement_ids.map(|id| (id, KindId(1))),
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .unwrap();
}

fn cold_replacement(
    execution: &ExecutionRequest<'_, '_>,
    original: &crate::runtime::RelationalRuntime,
    retired_roles: Vec<RequiredOutputRoleTarget>,
    replacement_ids: [EntityId; 2],
) {
    let checkpoint = original
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    runtime
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    let receipt = runtime
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let basis = test_owner_main_basis(&runtime).unwrap();
    let meaning = crate::mvcc::native_meaning_fixture("retirement-creator");
    let denied = runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "retirement-preserver",
            &[2; 32],
            &meaning,
            &retired_roles,
        )
        .unwrap_err();
    assert!(
        format!("{denied:?}").contains("superseded complete family publication"),
        "{denied:?}"
    );
    let before = checkpoint_manifest(
        &runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap(),
    );
    // Existing spellings still request fresh native allocations. Equal keys
    // cannot authorize Birth, and ordinary key interning is not this court.
    let (batch, targets) = creates(["new-left", "new-right"]);
    let denied = publish(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::Birth {
            admission: owner.admit_required_output("unrelated-birth", [8; 32]),
            absence: None,
            meaning: meaning.clone(),
            roles: new_roles(&targets),
        },
    )
    .unwrap_err();
    assert!(
        format!("{denied:?}").contains("cold route-absence readmission"),
        "{denied:?}"
    );
    let absence = runtime
        .readmit_required_output_route_absence(execution, &installed, &basis, &meaning)
        .unwrap();
    assert!(
        absence.is_none(),
        "a published route requires its exact predecessor"
    );
    let mut after = checkpoint_manifest(
        &runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap(),
    );
    // Phase-2 D7 records spent draft identities even when publication refuses.
    // Every other committed directory fact must remain byte-for-byte equal.
    assert_ne!(after.record_identity, before.record_identity);
    assert!(after.symbol_identity_frontier >= before.symbol_identity_frontier);
    after.record_identity = before.record_identity.clone();
    after.symbol_identity_frontier = before.symbol_identity_frontier;
    assert_eq!(after, before);
    let roles = replacement_ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| RequiredOutputRoleTarget {
            role: format!("created-{index}"),
            entity_name: "native-unit-output".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Create,
            target: EntityReference::Existing(id),
        })
        .collect::<Vec<_>>();
    let current = runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "retirement-preserver",
            &[3; 32],
            &meaning,
            &roles,
        )
        .unwrap();
    let renewed = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("cold-exact-Q-to-R"),
        preserve(&owner, current, replacement_ids, 4),
    )
    .unwrap();
    assert!(renewed.changed_records.is_empty());
    release_test_commit_snapshot(&runtime, &renewed);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let (batch, targets) = creates(["new-route-left", "new-route-right"]);
    let mut birth = native_birth_fixture(
        owner.admit_required_output("genuine-new-route", [9; 32]),
        targets.to_vec(),
    );
    if let RequiredOutputPublication::Birth {
        absence, meaning, ..
    } = &mut birth
    {
        *absence = runtime
            .readmit_required_output_route_absence(execution, &installed, &basis, meaning)
            .unwrap();
        assert!(absence.is_some());
    }
    let born = publish(&runtime, execution, &installed, batch, birth).unwrap();
    assert_eq!(born.changed_records.len(), 2);
    release_test_commit_snapshot(&runtime, &born);
}

fn checkpoint_manifest(
    checkpoint: &crate::durability::data::RelationalNativeCheckpoint,
) -> crate::durability::data::RelationalCheckpointManifest {
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    rmp_serde::from_slice::<Input>(
        checkpoint
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .expect("current native checkpoint framing"),
    )
    .unwrap()
    .manifest
}
