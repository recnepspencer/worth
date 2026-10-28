//! A real allocator reuse never reclassifies a stale logical coordinate.
use super::fixtures::*;
use crate::facade::runtime::PinnedAggregateReadDenialKind;
#[test]
fn isolated_carrier_birth_retirement_and_reused_generation_cover_both_cases() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(registry())
        .build()
        .expect("valid native runtime configuration");
    let born = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("isolated-carrier").push(entity(1, 0)),
    );
    let original = born.created_entity(&created(1, 0)).unwrap();
    let negative = read(execution, &runtime, &born.snapshot, key(original, 0));
    let positive = read(execution, &runtime, &born.snapshot, key(original, 1));
    assert_eq!(
        negative.members().cloned().collect::<Vec<_>>(),
        vec![key(original, 0)]
    );
    assert_eq!(
        positive.members().cloned().collect::<Vec<_>>(),
        vec![key(original, 1)]
    );
    let names = [negative.identity(), positive.identity()];
    assert_ne!(names[0], names[1]);
    drop(negative);
    drop(positive);
    runtime
        .visibility_authority()
        .release_snapshot(&born.snapshot)
        .unwrap();
    let removed = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("retire-both-isolated-cases").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: original,
            }),
        )),
    );
    let pin = runtime
        .read_truth()
        .pin_read_capability(&removed.snapshot, &[], execution)
        .unwrap();
    for side in 0..2 {
        let error = pin
            .read_projected_aggregate(
                execution,
                PinnedAggregateReadScope::for_member_definition(
                    descriptor().definition,
                    key(original, side),
                ),
            )
            .unwrap_err();
        assert_eq!(error.kind, PinnedAggregateReadDenialKind::MemberUnavailable);
    }
    drop(pin);
    runtime
        .visibility_authority()
        .release_snapshot(&removed.snapshot)
        .unwrap();
    runtime.retention().run_pass(execution).unwrap();
    let replacement = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("replacement-carrier").push(entity(1, 99)),
    );
    let actual = replacement.created_entity(&created(1, 99)).unwrap();
    assert_eq!(
        original.local_slot_value(),
        actual.local_slot_value(),
        "actual production allocator reused the retired carrier slot"
    );
    assert!(actual.generation_value() > original.generation_value());
    let pin = runtime
        .read_truth()
        .pin_read_capability(&replacement.snapshot, &[], execution)
        .unwrap();
    for side in 0..2 {
        assert_eq!(
            pin.read_projected_aggregate(
                execution,
                PinnedAggregateReadScope::for_member_definition(
                    descriptor().definition,
                    key(original, side)
                )
            )
            .unwrap_err()
            .kind,
            PinnedAggregateReadDenialKind::MemberUnavailable
        );
        let current = read(
            execution,
            &runtime,
            &replacement.snapshot,
            key(actual, side),
        );
        assert_eq!(
            current.members().cloned().collect::<Vec<_>>(),
            vec![key(actual, side)]
        );
        assert!(
            !names.contains(&current.identity()),
            "recreated carrier cases cannot reuse the retired generation's names"
        );
    }
}
