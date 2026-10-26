use super::*;
use crate::tests::support::*;

#[test]
fn shared_members_with_inconsistent_leaf_facts_refuse_the_manifest() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    create_entity_in_partition(
        &execution,
        &runtime,
        "shared",
        crate::identity::data::PartitionId(1),
    );
    let sibling = create_branch_from_main(&runtime, "other-root");
    create_entity_in_partition_on_branch(
        &execution,
        &runtime,
        "diverge",
        crate::identity::data::PartitionId(2),
        sibling,
    );
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: RelationalCheckpointManifest,
    }
    let input: Input = rmp_serde::from_slice(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let manifest = input.manifest;
    assert_eq!(manifest.roots.len(), 2);
    let first = &manifest.roots[0].regions[0];
    let shared = manifest.roots[1]
        .regions
        .iter()
        .position(|leaf| leaf.head == first.head && leaf.history == first.history)
        .expect("shared first partition");
    for mutation in 0..4 {
        let mut forged = manifest.clone();
        let leaf = &mut forged.roots[1].regions[shared];
        match mutation {
            0 => leaf.content_digest[0] ^= 1,
            1 => leaf.entity_slots += 1,
            2 => leaf.relation_slots += 1,
            _ => leaf.unavailable_records += 1,
        }
        let failure = validate(&forged).unwrap_err();
        assert_eq!(
            failure.class,
            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint
        );
    }
}
