//! Producer checkpoint and journal-prefix recovery fixture.
use super::*;

pub(super) fn fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    with_tail: bool,
) -> (
    RelationalDurableRecoverySource,
    Arc<MemberSource>,
    crate::identity::data::EntityId,
) {
    fixture_with_history(execution, with_tail, 0)
}

pub(super) fn fixture_with_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    with_tail: bool,
    updates: usize,
) -> (
    RelationalDurableRecoverySource,
    Arc<MemberSource>,
    crate::identity::data::EntityId,
) {
    fixture_from_runtime(
        execution,
        durable_runtime_with_test_schema(),
        with_tail,
        updates,
    )
}

pub(super) fn fixture_from_runtime(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: crate::runtime::RelationalRuntime,
    with_tail: bool,
    updates: usize,
) -> (
    RelationalDurableRecoverySource,
    Arc<MemberSource>,
    crate::identity::data::EntityId,
) {
    let entity = create_entity(execution, &runtime, "member-before");
    for update in 0..updates {
        update_entity_and_release_snapshot(
            execution,
            &runtime,
            entity,
            &format!("member-update-{update}"),
        );
    }
    checkpoint_fixture(execution, runtime, entity, with_tail)
}

pub(super) fn checkpoint_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: crate::runtime::RelationalRuntime,
    entity: crate::identity::data::EntityId,
    with_tail: bool,
) -> (
    RelationalDurableRecoverySource,
    Arc<MemberSource>,
    crate::identity::data::EntityId,
) {
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let covered = native.covered_position().unwrap();
    let transitions = native.covered_branch_transitions().unwrap();
    if with_tail {
        update_entity_and_release_snapshot(execution, &runtime, entity, "member-after");
    }
    let artifacts = attached_test_medium(&runtime).offered();
    let tail = artifacts
        .iter()
        .filter(|artifact| artifact.position() > covered)
        .cloned()
        .collect();
    let source = Arc::new(MemberSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("first capture owns every member")
                }
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|artifact| (artifact.position(), artifact.as_bytes().to_vec()))
            .collect(),
        damaged: AtomicBool::new(true),
        reads: AtomicUsize::new(0),
        member_reads: std::sync::Mutex::new(Vec::new()),
        commit_reads: std::sync::Mutex::new(Vec::new()),
        commits_damaged: AtomicBool::new(false),
        failed_member: std::sync::Mutex::new(None),
        failures: std::sync::Mutex::new(Default::default()),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            covered,
            transitions,
            native,
        )),
        tail,
        Vec::new(),
        source.clone(),
    );
    (recovery, source, entity)
}
