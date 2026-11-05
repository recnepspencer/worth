//! Direct custody contacts in the actual five-publication generator tree.
use super::{settlement::*, *};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn ordinary_root_intermediate_and_leaf_selectors_contact_only_direct_publications() {
    let app = world::install(2, Selections::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle(&app, &request), &request);
    let keys = ["root", "root.0", "root.0.0", "root.1", "root.1.0"];
    let sources = keys
        .into_iter()
        .map(|key| {
            (
                key,
                request.query(Read(key.into())).execute().unwrap().rows()[0].native_id,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let outputs = keys
        .into_iter()
        .map(|key| {
            (
                key,
                request
                    .query(realization_read::ReadRealization(key.into()))
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .native_id,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut all_rows = BTreeSet::new();
    let mut all_positions = BTreeSet::new();
    for key in keys {
        let contacts = app
            .runtime()
            .current_ownership_contacts_for_record_for_test(outputs[key], &scope)
            .unwrap();
        assert_eq!(
            contacts
                .publication_rows
                .iter()
                .map(|(source, _)| *source)
                .collect::<Vec<_>>(),
            [sources[key]],
            "output of {key}"
        );
        assert_eq!(
            contacts
                .presence_rows
                .iter()
                .map(|(source, _)| *source)
                .collect::<Vec<_>>(),
            [sources[key]],
            "position of {key}"
        );
        all_rows.extend(contacts.publication_rows);
        all_positions.extend(contacts.presence_rows);
    }
    assert_eq!(all_rows.len(), 5);
    assert_eq!(all_positions.len(), 5);
    for (seed, expected) in [
        (outputs["root"], vec![sources["root"]]),
        (sources["root.1"], vec![sources["root"], sources["root.1"]]),
        (
            sources["root.1.0"],
            vec![sources["root.1"], sources["root.1.0"]],
        ),
        (outputs["root.1.0"], vec![sources["root.1.0"]]),
    ] {
        let expected = expected.into_iter().collect::<BTreeSet<_>>();
        let reached = app
            .runtime()
            .current_ownership_sources_for_record_for_test(seed, &scope)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            reached, expected,
            "reviewer's direct-reach probe for {seed:?}"
        );
        let contacts = app
            .runtime()
            .current_ownership_contacts_for_record_for_test(seed, &scope)
            .unwrap();
        let expected_rows = all_rows
            .iter()
            .filter(|(owner, _)| expected.contains(owner))
            .copied()
            .collect::<BTreeSet<_>>();
        let expected_positions = all_positions
            .iter()
            .filter(|(owner, _)| expected.contains(owner))
            .copied()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            contacts
                .publication_rows
                .into_iter()
                .collect::<BTreeSet<_>>(),
            expected_rows,
            "no other roster row may be contacted"
        );
        assert_eq!(
            contacts.presence_rows.into_iter().collect::<BTreeSet<_>>(),
            expected_positions,
            "no other presence row may be contacted"
        );
    }
    for (source, expected) in [
        (sources["root"], keys.to_vec()),
        (sources["root.1"], vec!["root.1", "root.1.0"]),
    ] {
        let closure = app
            .runtime()
            .current_ownership_descendants_for_record_for_test(source, &scope)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            closure,
            expected.into_iter().map(|key| sources[key]).collect(),
            "the explicit cascade owner still reaches every descendant of {source:?}"
        );
    }
    // A field edit of an owned intermediate source checks its parent claim,
    // without reading the publications scoped by the intermediate itself.
    let reads = Arc::new(Mutex::new(Vec::new()));
    let sink = reads.clone();
    let guard = app
        .runtime()
        .record_current_ownership_contacts_for_test(&scope, move |contacts| {
            sink.lock().unwrap().push(contacts);
        })
        .unwrap();
    let result = request
        .advanced()
        .mutate(source_control::Control {
            key: "root.1".into(),
            width: 2,
        })
        .idempotency(&99)
        .execute_in_program::<Program>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    let actual = reads.lock().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(
        actual[0]
            .publication_rows
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        all_rows
            .iter()
            .filter(|(owner, _)| *owner == sources["root"])
            .copied()
            .collect()
    );
    assert_eq!(
        actual[0]
            .presence_rows
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        all_positions
            .iter()
            .filter(|(owner, _)| *owner == sources["root"])
            .copied()
            .collect()
    );
    drop(guard);
}
