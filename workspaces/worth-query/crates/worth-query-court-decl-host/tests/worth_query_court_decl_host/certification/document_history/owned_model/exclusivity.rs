//! Reparenting is validated through the affected descendants, not only its target.
use super::*;

#[test]
fn moving_a_single_owner_parent_refuses_a_descendant_with_two_roots() {
    let host = world::install_with_shared_descendant(true);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut conflicts = Vec::new();
    for preserve_previous in [true, false] {
        let outcome = request
            .advanced()
            .mutate(ownership_change::Change {
                from: "a".into(),
                to: "b".into(),
                member: "direct".into(),
                preserve_previous,
            })
            .without_source()
            .idempotency(&format!("shared:{preserve_previous}"))
            .execute_in_program(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )) = outcome
        else {
            panic!("{outcome:?}")
        };
        conflicts.push(
            cause
                .model_ownership_conflict()
                .expect("actual ownership conflict"),
        );
    }
    // The first invalid edge gives the parent two roots. Removing its old edge
    // leaves that parent singly owned, but its shared descendant still conflicts.
    for (conflict, key) in conflicts.iter().zip(["direct", "nested"]) {
        assert_eq!(
            (
                conflict.subject.partition_id.as_u32(),
                conflict.subject.local_slot_value(),
                conflict.subject.generation_value()
            ),
            world::member_identity(&host, host.current_world(), key)
        );
    }
    assert_eq!(
        world::member_value(&host, host.current_world(), "nested"),
        22
    );
}
