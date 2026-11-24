//! Phase 5E slice 2: held names are ordinary values. A derived record stores
//! a `NamedTarget` and a `WorthQueryName` as columns, an installed action
//! carries them as input fields, a later decision reads them back and
//! resolves them, and a derived projection returns each row's own name in
//! the read that returns the row.
use super::history::{named, navigate, record_edit, resolution, Navigate};
use super::named_read::{MaybeNamedModel, NamedMember};
use super::outputs::sculpt::{Plan, Step};
use super::outputs::{birth, keys, piece, read, resolved_to, World};
use super::*;
use probe::Observation;
use worth_query_decl::facade::application_naming::NamedTarget;

type HoldAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    stored::Binding,
    Completion,
>;

/// Whether a probe observed anything the model's record holds.
fn holds_nothing(observed: &[Observation]) -> bool {
    !observed.iter().any(|observation| match observation {
        Observation::ResolvedTo { label, .. }
        | Observation::Retired { label }
        | Observation::Ambiguous { label, .. }
        | Observation::ResolveRefused { label, .. } => label.starts_with("stored-"),
        _ => false,
    })
}

#[test]
fn a_record_holds_a_member_by_name_through_edits_and_history() {
    let token = "stored-names";
    let keys = ["direct", "nested"];
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);

    let (document, observed) = history::probe(&host, document, &request, token, &keys, 0);
    let (direct, nested) = (named(&observed, "direct"), named(&observed, "nested"));
    assert!(holds_nothing(&observed), "{observed:?}");

    // The action carries both names as input fields; the record stores them.
    let document = record_edit!(
        &host,
        document,
        &request,
        stored::Intent {
            target: NamedTarget::exact(direct),
            anchor: Some(nested),
        },
        HoldAction,
        "hold-direct"
    );
    // The probe rewrites another column of the same record: the held names
    // survive that edit and resolve to the members they were read from.
    let (document, observed) = history::probe(&host, document, &request, token, &keys, 1);
    let held_direct = document.current_step(&request).unwrap();
    assert_eq!(
        resolution(&observed, "stored-target"),
        &resolved_to("stored-target", "direct")
    );
    assert_eq!(
        resolution(&observed, "stored-anchor"),
        &resolved_to("stored-anchor", "nested")
    );

    // Holding another target with another selection replaces both columns.
    let document = record_edit!(
        &host,
        document,
        &request,
        stored::Intent {
            target: NamedTarget::all_descendants(nested),
            anchor: None,
        },
        HoldAction,
        "hold-nested"
    );
    let (document, observed) = history::probe(&host, document, &request, token, &keys, 2);
    assert_eq!(
        resolution(&observed, "stored-target"),
        &resolved_to("stored-target", "nested")
    );
    assert!(
        !observed.iter().any(|observation| matches!(
            observation,
            Observation::ResolvedTo { label, .. } if label == "stored-anchor"
        )),
        "a cleared anchor holds nothing: {observed:?}"
    );

    // History checkout restores the record, and with it the earlier names.
    navigate(
        &document,
        &request,
        Navigate::Checkout(held_direct.id()),
        0x5e2_0001,
    );
    let (_, observed) = history::probe(&host, document, &request, token, &keys, 3);
    assert_eq!(
        resolution(&observed, "stored-target"),
        &resolved_to("stored-target", "direct")
    );
    assert_eq!(
        resolution(&observed, "stored-anchor"),
        &resolved_to("stored-anchor", "nested")
    );
}

#[test]
fn a_stored_exact_target_refuses_ambiguous_once_its_member_splits() {
    let token = "stored-split";
    let world = World::install();
    world.committed(
        &format!("{token}-seed"),
        Plan {
            creates: keys(&["base"]),
            steps: vec![Step::Birth("base".into(), birth("base", "authored"))],
            ..Plan::default()
        },
    );
    let base = named(&world.probe(token, &["base"], 0), "base");
    world.hold(token, NamedTarget::exact(base));
    let observed = world.probe(token, &["base"], 1);
    assert_eq!(
        resolution(&observed, "stored-target"),
        &resolved_to("stored-target", "base")
    );

    world.committed(
        &format!("{token}-split"),
        Plan {
            reads: keys(&["base"]),
            deletes: keys(&["base"]),
            creates: keys(&["n1", "n2"]),
            steps: vec![Step::Split(
                read("base"),
                vec![
                    ("n1".into(), piece("cut", 1, None)),
                    ("n2".into(), piece("cut", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    let observed = world.probe(token, &["n1", "n2"], 2);
    let mut pieces = vec![named(&observed, "n1"), named(&observed, "n2")];
    pieces.sort();
    assert_eq!(
        resolution(&observed, "stored-target"),
        &Observation::Ambiguous {
            label: "stored-target".into(),
            candidates: pieces,
        },
        "the stored exact target never picks one piece"
    );
}

#[test]
fn a_read_projects_each_rows_own_persistent_name() {
    let token = "projected-names";
    let world = World::install();
    world.committed(
        &format!("{token}-seed"),
        Plan {
            creates: keys(&["base"]),
            steps: vec![Step::Birth("base".into(), birth("base", "authored"))],
            ..Plan::default()
        },
    );
    let observed = world.probe(token, &["direct", "base"], 0);
    let (direct, base) = (named(&observed, "direct"), named(&observed, "base"));
    let request = world.request();
    let on_world = request.on_branch(world.host.current_world());

    // A seeded member and one an operation named: the row carries the same
    // admitted name a decision reads with `name_of`.
    for (key, name) in [("direct", direct), ("base", base)] {
        let read = on_world
            .query(named_read::member::Read(key.into()))
            .execute()
            .unwrap_or_else(|cause| panic!("{key}: {cause:?}"));
        assert_eq!(
            read.rows(),
            [NamedMember {
                key: key.into(),
                name,
            }]
        );
    }

    // A model's kind has no name scheme: an optional name slot is absent and
    // a required one refuses typed instead of inventing a name.
    let read = on_world
        .query(named_read::model::Read("a".into()))
        .execute()
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    assert_eq!(
        read.rows(),
        [MaybeNamedModel {
            key: "a".into(),
            name: None,
        }]
    );
    let refused = on_world
        .query(named_read::strict_model::Read("a".into()))
        .execute();
    let rendered = format!(
        "{:?}",
        refused.err().expect("an unnamed record has no name")
    );
    assert!(rendered.contains("PersistentNameMissing"), "{rendered}");
}
