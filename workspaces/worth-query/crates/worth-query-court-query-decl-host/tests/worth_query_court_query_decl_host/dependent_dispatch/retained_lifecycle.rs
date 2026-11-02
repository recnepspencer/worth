//! Retained discovery composes with recovery, fork ancestry and another owner.
use super::split_support::*;
use super::*;

fn retain_root(mut operation: Operation, request: &Request<'_, '_, '_>) -> Operation {
    match operation.advance(request) {
        WorthQueryProgramOutputOperationAdvance::Pending(next) => {
            operation = next;
        }
        _ => panic!("the root must hand off before its dependent tree"),
    }
    operation
}

#[test]
fn retained_discovery_reuses_recovered_outputs_after_an_equal_source_commit() {
    // Original: root I, child I, leaf I. Reopen and retain root, then advance
    // the head by an equal authored source commit: all recovered outputs reuse,
    // with zero producer selections and the original full child output ID.
    let app = world::install(1, Selections::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let first = settle(&app, &request);
    let id = child_id(&first, "root.0");
    close(first, &request);
    let checkpoint = app
        .with_host_advancement(|phase| {
            app.capture_application_checkpoint(
                &phase,
                installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    drop((principal, scope, app));
    let selections = Selections::default();
    let app = world::restore(checkpoint, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let older = retain_root(start(&app, &request), &request);
    let retained = older.source_observation().selected_commit().clone();
    edit(&app, &request, "root", 1, 1, 301);
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        &retained
    );
    let current = finish(older, &request);
    assert_eq!(child_id(&current, "root.0"), id);
    assert!(selections.lock().unwrap().is_empty());
    assert_eq!(value(&request, "root.0.0"), 13);
    close(current, &request);
}

#[test]
fn post_fork_parent_publication_is_invisible_to_the_forks_retained_tree() {
    // Root I creates the child source before the fork, so both branches inherit
    // that exact source. Parent then publishes child I, leaf I. Fork selects root
    // P with its pre-fork prior, then child I, leaf I without a post-fork prior.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let parent = retain_root(start(&app, &request), &request);
    let branch = app
        .with_host_advancement(|phase| {
            app.branches()
                .fork(app.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap()
        .unwrap();
    let fork_request = app.request(&principal, &scope).on_branch(branch);
    let original = finish(parent, &request);
    let parent_id = child_id(&original, "root.0");
    let forked = settle(&app, &fork_request);
    assert_ne!(child_id(&forked, "root.0"), parent_id);
    sequence(&selections, "root", &[(false, false), (true, true)]);
    for key in ["root.0", "root.0.0"] {
        sequence(&selections, key, &[(false, false), (false, false)]);
    }
    assert_eq!(value(&request, "root.0.0"), 13);
    assert_eq!(value(&fork_request, "root.0.0"), 13);
    original.deliver_current(&request).unwrap();
    close(original, &request);
    close(forked, &fork_request);
}

#[test]
fn an_older_retained_operation_meets_the_newer_operations_preserve_row() {
    // Older retains root I before child production. Newer: child I -> P (its
    // owned absent cell), leaf I. Older rejoins that P row without another run.
    let selections = Selections::default();
    let app = world::install_with_cell(1, selections.clone(), true);
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let older = retain_root(start(&app, &request), &request);
    sequence(&selections, "root", &[(false, false)]);
    let newer = settle(&app, &request);
    let id = child_id(&newer, "root.0");
    let completed = finish(older, &request);
    assert_eq!(child_id(&completed, "root.0"), id);
    sequence(&selections, "root", &[(false, false)]);
    sequence(&selections, "root.0", &[(false, false), (true, true)]);
    sequence(&selections, "root.0.0", &[(false, false)]);
    newer.deliver_current(&request).unwrap();
    close(newer, &request);
    close(completed, &request);
}
