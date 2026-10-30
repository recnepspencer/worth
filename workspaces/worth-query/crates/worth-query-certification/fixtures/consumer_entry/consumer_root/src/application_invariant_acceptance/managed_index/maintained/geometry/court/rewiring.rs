//! Rewiring moves the dependency; two physical edges to one endpoint remain two.
use super::*;
pub(super) fn run(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    calls: &Arc<provider::Calls>,
    rules: &Arc<rule::Calls>,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    commit(
        world,
        principal,
        scope,
        Edit::RewireEnd {
            directory: "A".into(),
            edge: "e1".into(),
            end: "v5".into(),
        },
        0x919412,
    );
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(find(directory, 25)).execute().unwrap().rows()),
            expected(directory, 30, find(directory, 25).probe)
        );
    }
    rules.records.lock().unwrap().clear();
    let updates = calls.updates.load(Ordering::SeqCst);
    // v1 was the old end. Its movement must not refit the rewired edge or run
    // the Source-scoped rule through a stale reverse dependency posting.
    commit(
        world,
        principal,
        scope,
        Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 50,
            y: 0,
        },
        0x919413,
    );
    assert_eq!(calls.updates.load(Ordering::SeqCst), updates);
    assert!(rules.records.lock().unwrap().is_empty());
    assert_eq!(
        values(request.query(find("A", 25)).execute().unwrap().rows()),
        expected("A", 30, find("A", 25).probe)
    );
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let duplicate = request
        .advanced()
        .mutate(Edit::DuplicateEnd {
            directory: "A".into(),
            edge: "e1".into(),
        })
        .idempotency(&0x919414u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = duplicate
    else {
        panic!("two physical Ends must violate the actual Source cardinality: {duplicate:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    let native = denial
        .invariant_execution_failure()
        .unwrap()
        .managed_source_denial()
        .expect("the original Source preparation failure is retained");
    let provider = native
        .provider_failure()
        .expect("the Source projection's typed failure survived Native");
    let mut cause: &(dyn std::error::Error + 'static) = provider.cause();
    loop {
        if let Some(projection) = cause.downcast_ref::<WorthQueryApplicationProjectionDenial>() {
            assert_eq!(
                projection.kind(),
                WorthQueryApplicationProjectionDenialKind::RelationCardinalityMismatch
            );
            break;
        }
        cause = cause
            .source()
            .expect("the original public projection denial remains in the owned chain");
    }
    assert!(
        rules.records.lock().unwrap().is_empty(),
        "cardinality fails during staging before a domain callback"
    );
    assert_eq!(
        calls.updates.load(Ordering::SeqCst),
        updates,
        "no callback updates a root from a malformed Source"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        values(request.query(find("B", 25)).execute().unwrap().rows()),
        expected("B", 30, find("B", 25).probe)
    );
}
