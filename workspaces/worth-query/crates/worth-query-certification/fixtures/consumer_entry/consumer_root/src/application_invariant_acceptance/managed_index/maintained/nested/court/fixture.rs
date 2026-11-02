//! All graph edits use installed conventional authoring and actual publication.
use super::*;
pub(super) fn publish(world: &host::Host, model: &mut Model, key: &mut u64, edit: Edit) {
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    *key += 1;
    let outcome = request
        .advanced()
        .mutate(edit.clone())
        .idempotency(key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "ordinary authored edit {edit:?}: {outcome:?}"
    );
    model.apply(&edit);
}
pub(super) fn setup() -> (host::Host, Arc<prefix::Calls>, Arc<rule::Calls>, Model, u64) {
    let calls = Arc::new(prefix::Calls::default());
    let records = Arc::new(rule::Calls::default());
    let world = host::install(Arc::clone(&calls), Arc::clone(&records));
    let mut model = Model::default();
    model.apply(&Edit::Directory {
        identity: "catalog".into(),
    });
    let mut key = 0x9196_0000;
    for directory in ["A", "B", "D"] {
        records.records.lock().unwrap().clear();
        publish(
            &world,
            &mut model,
            &mut key,
            Edit::Directory {
                identity: directory.into(),
            },
        );
        let observed = records.records.lock().unwrap();
        let new: Vec<_> = observed
            .iter()
            .filter(|record| record.before.is_none() && record.after.is_empty())
            .collect();
        assert_eq!(
            new.len(),
            1,
            "the genuinely new empty Source scope must be observed"
        );
        // A real owner-issued locator, never synthesized from the semantic label.
        // All graph membership/value expectations still come from authored edits.
        model.record_scope(directory, new[0].scope);
    }
    for (directory, identity, name, root) in [
        ("A", "x", "match-root", true),
        ("B", "y", "other-root", true),
        ("D", "w", "match-unrelated", true),
        ("catalog", "a", "branch-a", false),
        ("catalog", "b", "branch-b", false),
        ("catalog", "z", "shared-leaf", false),
        ("catalog", "c", "tail", false),
        ("catalog", "q", "ancestor", false),
        ("catalog", "p", "peer", false),
    ] {
        publish(
            &world,
            &mut model,
            &mut key,
            Edit::Item {
                directory: directory.into(),
                identity: identity.into(),
                name: name.into(),
                root,
                incarnation: 1,
            },
        );
    }
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Share {
            directory: "B".into(),
            item: "x".into(),
        },
    );
    records.records.lock().unwrap().clear();
    (world, calls, records, model, key)
}
pub(super) fn link(
    world: &host::Host,
    model: &mut Model,
    key: &mut u64,
    from: &str,
    to: &str,
    edition: u64,
) {
    publish(
        world,
        model,
        key,
        Edit::Link {
            from: from.into(),
            to: to.into(),
            edition,
        },
    );
}
pub(super) fn find(scope: &str, prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: scope.into(),
        prefix: prefix.into(),
    }
}
