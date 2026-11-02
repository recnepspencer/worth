//! Opening's real provider read overlaps an independently admitted publication.
use super::*;
use std::{sync::mpsc, time::Duration};

pub(super) fn run(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<super::super::super::schema::Catalog>,
    scope: &WorthQueryRequestScope,
    calls: &Arc<prefix::Calls>,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let bracket = request
        .advanced()
        .mutate(Edit::Item {
            directory: "A".into(),
            identity: "race-bound".into(),
            name: "zz-bound".into(),
        })
        .idempotency(&0x9198_02ffu64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            bracket,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{bracket:?}"
    );
    // race lies strictly inside the genuine later..zz-bound range. The empty
    // search reaches a real predicate without an artificial summary or pause.
    let empty = request.query(find("A", "race")).execute().unwrap();
    assert!(empty.rows().is_empty());
    let source = request.query(find("A", "later")).execute().unwrap();
    let expectation = source.result_set_observation().unwrap().clone();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (released_tx, released_rx) = mpsc::sync_channel(1);
    calls.matched_sources.store(1, Ordering::SeqCst);
    *calls.after_second_match.lock().unwrap() = Some(Box::new(move || {
        entered_tx.send(()).unwrap();
        released_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("independent real writer completes while the selected read is held");
    }));
    let opening = std::thread::scope(|threads| {
        let writer = threads.spawn(move || {
            entered_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("opening reached the real provider row predicate");
            let scope = authentication::request_scope();
            let principal = authentication::block_on(
                authentication::admit(world.installed_schema()).authenticate(
                    authentication::LocalCredential::issued_for_model_owner(),
                    &scope,
                ),
            )
            .unwrap();
            let changed = world
                .runtime()
                .request(&principal, &scope)
                .on_branch(world.current_world())
                .advanced()
                .mutate(rename("later", "race"))
                .expect_result_set(expectation)
                .idempotency(&0x9198_0300u64)
                .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation);
            released_tx.send(()).unwrap();
            let changed = changed.unwrap();
            assert!(
                matches!(
                    changed,
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ),
                "{changed:?}"
            );
        });
        let opening = request
            .query(find("A", "race"))
            .watch_index::<prefix::PrefixIndex>();
        writer.join().unwrap();
        opening
    });
    let refusal = opening
        .err()
        .expect("an old empty read cannot register after a matching entrant");
    assert!(
        matches!(refusal,WorthQueryApplicationRequestQueryDenial::ManagedIndex(ref cause)
        if matches!(cause.cause(),WorthQueryManagedIndexDenialCause::Product(WorthQueryProductBranchAdmissionDenial::Refused(WorthQueryProductBranchAdmissionRefusalKind::ObservationStaleSourceHead)))),
        "{refusal:?}"
    );
    assert!(
        empty.rows().is_empty(),
        "the original genuine empty image stays immutable"
    );
    let (current, watch) = request
        .query(find("A", "race"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(values(current.rows()), vec![("x".into(), "race".into())]);
    watch.close().unwrap();
}
