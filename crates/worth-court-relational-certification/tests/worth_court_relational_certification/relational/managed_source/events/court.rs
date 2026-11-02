//! Noncommuting event semantics and actual before/after changed-row cohorts.
use super::*;
use fixture::*;
#[test]
fn ordered_event_source_preserves_newer_introductions_and_same_step_retirement() {
    with_event_source(|runtime, control, court, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let [a, b, c, d, u] = w.events;
        let [x, y, z] = w.rows;
        let full: [&[usize]; 5] = [&[0, 1], &[1], &[0, 1], &[0, 1], &[2]];
        expect(control, court, &w, [&[]; 5], full, &[0, 1, 2, 3, 4]);
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("ordered-event-history")
                .push(link(5, b, a, "ba"))
                .push(link(5, c, b, "cb"))
                .push(link(5, d, c, "dc"))
                .push(link(2, a, x, "ax"))
                .push(link(2, a, y, "ay"))
                .push(link(3, b, x, "bx-retired"))
                .push(link(2, c, x, "cx-reintroduced"))
                .push(link(2, d, z, "dz-introduced"))
                .push(link(3, d, z, "dz-retired"))
                .push(link(2, u, z, "uz")),
        );
        assert_scopes(court, &w, &[0, 1, 2, 3, 4]);
        assert!(
            control.checked.load(Ordering::SeqCst) >= 5,
            "actual forward/inverse observations ran"
        );
        let after: [&[usize]; 5] = [&[0, 1], &[1], &[1], &[1], &[2]];
        expect(control, court, &w, full, after, &[0, 1, 2, 3]);
        let removed = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("remove-newer-introduction").push(delete(relation(
                &born,
                2,
                c,
                x,
                "cx-reintroduced",
            ))),
        );
        assert!(
            removed.change_summary().unwrap().adjacency_delta_count > 0,
            "deleting the actual introduction must carry its physical adjacency effect: {:?}",
            removed.change_summary()
        );
        assert_scopes(court, &w, &[0, 1, 2, 3]);
        for event in [c, d] {
            assert_changed(court, event, &[x, y]);
        }
        // Deleting retirement makes x re-enter through its older introduction.
        expect(
            control,
            court,
            &w,
            after,
            [&[0, 1], &[0, 1], &[0, 1], &[0, 1], &[2]],
            &[0, 1, 2, 3],
        );
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("remove-retirement").push(delete(relation(
                &born,
                3,
                b,
                x,
                "bx-retired",
            ))),
        );
        assert_scopes(court, &w, &[0, 1, 2, 3]);
    });
}
#[test]
fn ordered_event_predecessor_rewire_replaces_old_and_new_prefix_members() {
    with_event_source(|runtime, control, court, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let [a, b, c, d, u] = w.events;
        let [x, _, z] = w.rows;
        let before: [&[usize]; 5] = [&[0], &[0], &[0], &[0], &[2]];
        expect(control, court, &w, [&[]; 5], before, &[0, 1, 2, 3, 4]);
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("rewire-history")
                .push(link(5, b, a, "ba"))
                .push(link(5, c, b, "cb"))
                .push(link(5, d, c, "dc"))
                .push(link(2, a, x, "ax"))
                .push(link(2, u, z, "uz")),
        );
        let after: [&[usize]; 5] = [&[0], &[0], &[2], &[2], &[2]];
        expect(control, court, &w, before, after, &[0, 1, 2, 3, 4]);
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("rewire-to-other-history")
                .push(delete(relation(&born, 5, c, b, "cb")))
                .push(link(5, c, u, "cu")),
        );
        assert_scopes(court, &w, &[0, 1, 2, 3, 4]);
        for event in [c, d] {
            assert_changed(court, event, &[x, z]);
        }
    });
}
