//! Full-generation selection and complete retained membership support.
use super::super::super::{member_delete, program, world};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

fn coordinate(
    id: worth_relational::facade::identity::EntityId,
) -> worth_foundational::facade::EntityId {
    worth_foundational::facade::EntityId::new(
        worth_foundational::facade::PartitionId(id.partition_value()),
        id.local_slot_value(),
        id.generation_value(),
    )
}

#[test]
fn exact_member_identity_stays_inside_scope_and_retains_membership_support() {
    let host = world::install_configured::<program::Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        1, Some(2), world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let all = request
        .query(super::super::members::Read("a".into()))
        .execute()
        .unwrap();
    assert!(all.rows()[0].members.len() > 1);
    let original = all.rows()[0]
        .members
        .iter()
        .find(|row| row.key == "m")
        .unwrap();
    let actual = coordinate(original.native_id);
    let read = |model: &str, entity| Read {
        model: model.into(),
        entity,
    };
    let selected = request.query(read("a", actual)).execute().unwrap();
    assert_eq!(selected.rows()[0].members.len(), 1);
    assert_eq!(selected.rows()[0].members[0].key, "m");
    assert_eq!(selected.rows()[0].members[0].native_id, original.native_id);
    assert!(
        request.query(read("b", actual)).execute().unwrap().rows()[0]
            .members
            .is_empty(),
        "an authentic coordinate must not become a lookup root outside Model b"
    );
    use worth_foundational::facade::{EntityId, PartitionId};
    for absent in [
        EntityId::new(actual.partition_id, actual.local_slot.0, 0),
        EntityId::new(
            actual.partition_id,
            actual.local_slot.0,
            actual.generation.0.checked_add(1).unwrap(),
        ),
        EntityId::new(actual.partition_id, u64::MAX, actual.generation.0),
        EntityId::new(
            PartitionId(u32::MAX),
            actual.local_slot.0,
            actual.generation.0,
        ),
    ] {
        assert!(
            request.query(read("a", absent)).execute().unwrap().rows()[0]
                .members
                .is_empty(),
            "the full coordinate is equality data, not a wildcard or Native direct lookup"
        );
    }
    let observed = selected.observed_sources()[0].clone();
    let accepted = request
        .advanced()
        .mutate(expectation::Observe(actual))
        .expect_source(observed.clone())
        .idempotency(&"identity-supported".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            accepted,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{accepted:?}"
    );
    let deleted = request
        .advanced()
        .mutate(member_delete::Intent)
        .idempotency(&"identity-remove-member".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            deleted,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{deleted:?}"
    );
    let empty = request.query(read("a", actual)).execute().unwrap();
    assert!(empty.rows()[0].members.is_empty());
    assert!(
        !request
            .query(super::super::members::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .members
            .is_empty(),
        "absence selects the exact coordinate, not an empty Model roster"
    );
    let head = super::super::super::synchronous_edit::native_head(&host);
    let refused = request
        .advanced()
        .mutate(expectation::Observe(actual))
        .expect_source(observed)
        .idempotency(&"identity-stale-support".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = refused
    else {
        panic!("{refused:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceRetired
    );
    assert_eq!(
        super::super::super::synchronous_edit::native_head(&host),
        head
    );
    let empty_observation = empty.observed_sources()[0].clone();
    let accepted = request
        .advanced()
        .mutate(expectation::Observe(actual))
        .expect_source(empty_observation)
        .idempotency(&"identity-empty-supported".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            accepted,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{accepted:?}"
    );
}
