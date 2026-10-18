//! Typed stops at pins are a declared new law, not a master differential.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::facade::indexes::BoundedEntityFieldSelectedLookup;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial as FieldDenial,
    BoundedEntityFieldLookupDenialKind as FieldKind, BoundedRelationJoinLookupDenial as JoinDenial,
    BoundedRelationJoinLookupDenialKind as JoinKind,
};
use crate::indexes::data::{
    BoundedIndexParityMode, BoundedRelationJoinLookupRequest, DerivedIndexId,
};
fn stopped_backings(mut run: impl FnMut(&ExecutionRequest<'_, '_>, Cause)) {
    for deadline in [false, true] {
        let source = worth_execution::CancellationSource::new();
        let deadline_at =
            deadline.then(|| std::time::Instant::now() - std::time::Duration::from_secs(1));
        let serial = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(MEMORY),
            source.token(),
            deadline_at,
        );
        let lease = test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(
                        std::num::NonZeroUsize::new(1).unwrap(),
                        MEMORY,
                        Some(100_000),
                    ),
                ),
                deadline: deadline_at,
                cancellation: source.token(),
            })
            .unwrap();
        if !deadline {
            source.cancel();
        }
        let cause = if deadline {
            Cause::DeadlineElapsed
        } else {
            Cause::Cancelled
        };
        run(&ExecutionRequest::serial(&serial), cause);
        run(&ExecutionRequest::leased(&lease), cause);
    }
}
#[test]
fn runtime_read_law_view_input_stops_and_refuses_by_value() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, true);
    let view = world
        .runtime
        .read_truth()
        .project_snapshot(&world.snapshot)
        .unwrap();
    // New pin-stop law: these rows intentionally are not run on bdb91675a9.
    stopped_backings(|request, cause| {
        let answer = world.runtime.index_access().prepare_entity_field_lookup(
            &view,
            world.index,
            KindId(1),
            &world.name,
            request,
        );
        assert_eq!(
            answer.err().unwrap(),
            FieldDenial::new(FieldKind::Execution(cause), world.index)
        );
    });
    let before = world.runtime.index_access().generation_selection_counters();
    let denied = world
        .runtime
        .index_access()
        .prepare_entity_field_lookup(&view, world.index, KindId(1), &world.label, &setup)
        .err()
        .unwrap();
    assert_eq!(
        denied,
        FieldDenial::new(FieldKind::WrongIndexKind, world.index)
    );
    assert_eq!(
        world.runtime.index_access().generation_selection_counters(),
        before
    );
}
#[test]
fn runtime_read_law_selected_view_stops_and_refuses_by_value() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, true);
    let view = world
        .runtime
        .read_truth()
        .project_snapshot(&world.snapshot)
        .unwrap();
    let value = string_aspect_value("a");
    let lookup = |request: &ExecutionRequest<'_, '_>, index| {
        world
            .runtime
            .index_access()
            .execute_bounded_entity_field_lookup(
                BoundedEntityFieldSelectedLookup::new(
                    &view,
                    index,
                    KindId(1),
                    &world.name,
                    &value,
                    2,
                    |_, _| Ok::<_, ()>(()),
                ),
                BoundedIndexParityMode::Production,
                request,
            )
    };
    // New pin-stop law: these rows intentionally are not run on bdb91675a9.
    stopped_backings(|request, cause| match lookup(request, world.index) {
        Err(Stop::Lookup(denial)) => assert_eq!(
            denial,
            FieldDenial::new(FieldKind::Execution(cause), world.index)
        ),
        other => panic!("expected the caller stop, got {other:?}"),
    });
    let missing = DerivedIndexId(u64::MAX);
    match lookup(&setup, missing) {
        Err(Stop::Lookup(denial)) => assert_eq!(
            denial,
            FieldDenial::new(FieldKind::IndexNotInstalled, missing)
        ),
        other => panic!("expected index absence, got {other:?}"),
    }
}
#[test]
fn runtime_read_law_relation_join_stops_and_refuses_by_value() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, true);
    let lookup = |request: &ExecutionRequest<'_, '_>| {
        world
            .runtime
            .index_access()
            .execute_bounded_relation_join_lookup(
                BoundedRelationJoinLookupRequest::new(
                    world.snapshot.clone(),
                    world.index,
                    world.entities[0],
                    world.entities[2],
                    2,
                )
                .unwrap(),
                BoundedIndexParityMode::Production,
                request,
            )
    };
    // New pin-stop law: these rows intentionally are not run on bdb91675a9.
    stopped_backings(|request, cause| {
        assert_eq!(
            lookup(request).unwrap_err(),
            JoinDenial::new(JoinKind::Execution(cause), world.index)
        )
    });
    let before = world.runtime.index_access().generation_selection_counters();
    assert_eq!(
        lookup(&setup).unwrap_err(),
        JoinDenial::new(JoinKind::CorruptIndexEntries, world.index)
    );
    assert_eq!(
        world.runtime.index_access().generation_selection_counters(),
        before
    );
}
