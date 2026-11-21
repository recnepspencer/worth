//! Checkpoints retain only the live derived-index payload across repeated edits.
//! Fixture adaptation (C2.3): the original checkpointed a runtime bound to the
//! retired in-crate test medium through `checkpoint_durably`, which also filed
//! a native image in that medium. The asserted sections are the owner's own
//! `DurableCheckpoint` and resident generations, so the claim has no
//! physical-medium content and runs on the Memory runtime.
use super::*;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};

fn name_index(runtime: &crate::runtime::RelationalRuntime) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "checkpoint-index-tail".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity")
}

fn build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    index: DerivedIndexId,
    outcome: &crate::transactions::data::CommitResult,
) -> crate::indexes::data::DerivedIndexGenerationId {
    build_on_branch(execution, runtime, index, outcome, BranchId("main".into()))
}

fn build_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    index: DerivedIndexId,
    outcome: &crate::transactions::data::CommitResult,
    branch_id: BranchId,
) -> crate::indexes::data::DerivedIndexGenerationId {
    let built = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id,
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty());
    built.generations[0].generation_id
}

#[test]
fn repeated_edits_checkpoint_only_the_live_index_payload() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "checkpoint-size-0");
    let index = name_index(&runtime);
    build(execution, &runtime, index.index_id, &first);
    release_test_commit_snapshot(&runtime, &first);
    for ordinal in 1..24 {
        let outcome =
            create_entity_outcome(execution, &runtime, &format!("checkpoint-size-{ordinal}"));
        build(execution, &runtime, index.index_id, &outcome);
        release_test_commit_snapshot(&runtime, &outcome);
    }
    let legacy_all = crate::indexes::data::DerivedIndexArtifacts::new(
        runtime.index_access().generations_snapshot(),
    );
    let legacy_bytes = rmp_serde::to_vec_named(&legacy_all).unwrap().len();
    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let retained_bytes =
        rmp_serde::to_vec_named(checkpoint.derived_index_checkpoint.as_ref().unwrap())
            .unwrap()
            .len();
    eprintln!(
        "index checkpoint sections: legacy_all={legacy_bytes} retained_delta={retained_bytes}"
    );
    assert!(retained_bytes * 4 < legacy_bytes);
    assert!(runtime.index_access().generations_snapshot().len() < 24);
}
