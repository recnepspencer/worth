//! Phase 5E slice 2: held names are ordinary values. A derived record stores
//! a `NamedTarget` and a `WorthQueryName` as columns, an installed action
//! carries them as input fields, a later decision reads them back and
//! resolves them, and a derived projection returns each row's own name in
//! the read that returns the row.
use super::named_read::{MaybeNamedModel, NamedMember};
use super::outputs::sculpt::{Plan, Step};
use super::outputs::{birth, keys, named, piece, read, resolution, resolved_to, World};
use super::*;
use probe::Observation;
use worth_query_decl::facade::application_naming::NamedTarget;

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

#[test]
fn a_history_capture_projects_each_rows_own_persistent_name() {
    use std::num::NonZeroUsize;
    let token = "history-capture-projected-names";
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
    let runtime = world.host.runtime();
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    let limits = WorthQueryApplicationHistoryCaptureLimits {
        maximum_subjects: NonZeroUsize::MIN,
        maximum_aspects: NonZeroUsize::MIN,
        maximum_source_visits: NonZeroUsize::new(16).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(64 * 1024).unwrap(),
    };
    for (key, expected) in [("direct", direct), ("base", base)] {
        let rows = runtime
            .with_application_advancement(&scope, |phase| {
                let history = runtime
                    .installed_schema()
                    .installed_history_query::<named_read::MemberHistory>()
                    .expect("MemberHistory is installed");
                let selected = runtime
                    .on_branch(runtime.current_world())
                    .select(&phase)
                    .unwrap();
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        history.subject_read().principal_binding(),
                        &external,
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let entity = selected
                    .resolve_entity(
                        &phase,
                        MemberKey::at("MemberKey"),
                        key.to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let access = WorthQueryApplicationQueryAccessContext::new(&principal, &entity);
                let plan = selected
                    .admit_application_query(
                        &phase,
                        history.subject_read().query(),
                        &access,
                        ApplicationQueryParameterSet::new(),
                        WorthQueryProductQueryControls::new(NonZeroUsize::MIN, &scope),
                    )
                    .unwrap();
                let execution = phase
                    .checked_execution_request_for(runtime.product_runtime())
                    .unwrap();
                let capture = runtime
                    .admit_application_history_capture(&execution, &history, plan, limits)
                    .unwrap();
                let (read, _) = runtime
                    .execute_application_history_capture(&phase, capture)
                    .unwrap_or_else(|cause| panic!("{key}: {cause:?}"))
                    .into_parts();
                read.rows().to_vec()
            })
            .unwrap();
        assert_eq!(
            rows,
            vec![NamedMember {
                key: key.to_owned(),
                name: expected,
            }]
        );
    }
}
