//! A genuine node from an unrelated build cannot enter the inherited page edit.
use super::*;

pub(in super::super::super) fn run() {
    let (world, calls, _, mut key) = setup();
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let unrelated = request
        .query(find("U", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(find("U", ""))
        .rebuild_index(&unrelated)
        .unwrap();
    let foreign = calls
        .observed_root
        .lock()
        .unwrap()
        .expect("genuine unrelated node");
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    *calls.foreign_node.lock().unwrap() = Some(foreign);
    key += 1;
    let result = request
        .advanced()
        .mutate(append("E", "A", Some(("e".into(), "match-e".into())), None))
        .without_source()
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("foreign node must refuse: {result:?}")
    };
    let source = denial
        .invariant_execution_failure()
        .unwrap()
        .managed_source_denial()
        .unwrap();
    let mut cause: &(dyn std::error::Error + 'static) = source.provider_failure().unwrap().cause();
    loop {
        if let Some(work) = cause.downcast_ref::<WorthQueryManagedIndexWorkDenial>() {
            assert!(
                matches!(
                    work,
                    WorthQueryManagedIndexWorkDenial::Controlled(
                        WorthQueryManagedIndexStop::ForeignNode
                    )
                ),
                "{work:?}"
            );
            break;
        }
        cause = cause.source().expect("original typed provider stop");
    }
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        names(request.query(find("A", "")).execute().unwrap().rows()),
        set(&["x"])
    );
    *calls.foreign_node.lock().unwrap() = None;
    edit_history(
        &world,
        &mut key,
        append("E", "A", Some(("e".into(), "match-e".into())), None),
    );
    assert_eq!(
        names(request.query(find("E", "")).execute().unwrap().rows()),
        set(&["x", "e"])
    );
}
