//! Same-carrier adoption keeps native payloads and rebuilds derived indexes.
use super::{
    fixture::{self, *},
    recovered,
    retained_records::RetainedRecords,
    serial_request, RelationalOperationControl,
};
use worth_relational::facade::{identity::KindId, indexes::DerivedIndexEntries};

#[test]
fn genuine_native_codec_adoption_preserves_raw_source_and_rebuilds_target_equality_index() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let source_before = protected_payload(execution, &world.runtime, world.authored);
    let source_carrier = world
        .runtime
        .config()
        .schema
        .registry
        .entity_registration(KindId(1))
        .unwrap()
        .clone();
    assert!(!source_before.is_empty());
    let old_generation = world
        .runtime
        .index_access()
        .read_latest_generation(
            execution,
            INDEX,
            basis(&world.runtime).descriptor().branch_id(),
        )
        .unwrap()
        .unwrap();
    assert!(
        matches!(&old_generation.entries,DerivedIndexEntries::EntityField(entries) if !entries.is_empty())
    );
    let (mut target, mut recovered, receipt) = recovered(execution, &world);
    assert_eq!(
        protected_payload(execution, &target, world.authored),
        source_before
    );
    assert_eq!(
        target
            .config()
            .schema
            .registry
            .entity_registration(KindId(1)),
        Ok(&source_carrier)
    );
    assert!(target
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .is_err());
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    let selected = basis(&target);
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap();
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let committed = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap();
    let (commit, recovered) = committed.into_parts();
    assert!(!recovered.codec_transition_pending());
    assert!(recovered.codec_transition_retires(world.output));
    assert!(!recovered.codec_transition_retires(world.authored));
    let target_receipt = target
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_eq!(
        protected_payload(execution, &target, world.authored),
        source_before
    );
    assert_eq!(
        target
            .config()
            .schema
            .registry
            .entity_registration(KindId(1)),
        Ok(&source_carrier)
    );
    let current = basis(&target);
    let retired = RetainedRecords::capture(execution, &target);
    assert!(!retired.entity(world.output).0);
    let target_generation = target
        .index_access()
        .read_latest_generation(execution, INDEX, current.descriptor().branch_id())
        .unwrap()
        .unwrap();
    assert_eq!(target_generation.source_commit_id, commit.commit_id);
    assert_ne!(
        target_generation.generation_id,
        old_generation.generation_id
    );
    assert!(
        matches!(&target_generation.entries,DerivedIndexEntries::EntityField(entries) if entries.is_empty())
    );
    recovered.admit_basis(current).unwrap();
    let (producer, installed) = target_receipt.issue_application_publication_owner();
    let regenerated = create(
        execution,
        &target,
        3,
        "derived-v2",
        r#"["new",41]"#,
        Some((
            &installed,
            producer.admit_required_output("native.fixture.producer", [10; 32]),
        )),
    );
    build_index(execution, &target);
    let rebuilt = target
        .index_access()
        .read_latest_generation(execution, INDEX, basis(&target).descriptor().branch_id())
        .unwrap()
        .unwrap();
    assert!(
        matches!(&rebuilt.entries,DerivedIndexEntries::EntityField(entries)
        if entries.len()==1 && entries.iter().any(|(_,ids)| ids.iter().any(|id| *id == regenerated)))
    );
    assert_eq!(
        protected_payload(execution, &target, world.authored),
        source_before
    );
    let saved = capture(execution, &target);
    let (mut reopened, receipt) = fixture::installed(2);
    reopened
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &saved, worth_testkit_relational::cold_recovery::members_source(&saved))
        .unwrap();
    reopened
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_eq!(
        protected_payload(execution, &reopened, world.authored),
        source_before
    );
    assert_eq!(
        protected_payload(execution, &reopened, regenerated),
        protected_payload(execution, &target, regenerated)
    );
    let old = basis(&reopened);
    let saved_basis = basis(&target);
    let before = saved_basis.reference().target().as_basis().unwrap();
    let after = old.reference().target().as_basis().unwrap();
    assert_ne!(after.runtime_instance_id(), before.runtime_instance_id());
    assert_eq!(after.selected_commit_id(), before.selected_commit_id());
    assert_eq!(after.version_id(), before.version_id());
    assert_eq!(after.parent_commit_ids(), before.parent_commit_ids());
    assert_eq!(after.roots().truth_root(), before.roots().truth_root());
    assert_eq!(after.roots().schema_root(), before.roots().schema_root());
    let reopened_records = RetainedRecords::capture(execution, &reopened);
    assert!(!reopened_records.entity(world.output).0);
    assert_eq!(
        reopened_records.entity(world.output).1,
        RetainedRecords::capture(execution, &target)
            .entity(world.output)
            .1
    );
    let reopened_index = reopened
        .index_access()
        .read_latest_generation(execution, INDEX, old.descriptor().branch_id())
        .unwrap()
        .unwrap();
    assert_eq!(reopened_index.entries, rebuilt.entries);
    assert_eq!(reopened_index.source_commit_id, rebuilt.source_commit_id);
}
