//! Phase 5E slice 3B-1: a naming scope binds one authored occurrence for
//! every birth and posture under it. Scoped authoring mints the names the
//! manual candidate-writer calls mint, and a least-keeper merge keeps the
//! least parent name whatever order its parents are listed in.
use super::outputs::sculpt::{HeldSeam, Plan, ScopeStep, Step};
use super::outputs::seam_names::{ends, link, members, read_name, resolve, seam, seam_read};
use super::outputs::{
    birth, exact, keys, named, naming_refusal, read, resolution, resolved_to, sculpt, seam_between,
    CutSide, World,
};
use super::*;
use worth_query_host::facade::application_naming::{
    LineageNameDenial, LineagePostureKind, OccurrencePath, WorthQueryName, WorthQueryOutputBirth,
};

fn occurrence(path: &str) -> OccurrencePath {
    OccurrencePath::parse(path).expect("a valid occurrence")
}

/// Every name the authoring court checks: members `a..d` and seam `s`.
fn authored_names(world: &World, token: &str) -> Vec<[u8; 32]> {
    probe::seams::name(token, "s", "a", "b");
    let observed = world.probe(token, &["a", "b", "c", "d"], 0);
    let seams = probe::seams::take(token);
    let mut names: Vec<[u8; 32]> = ["a", "b", "c", "d"]
        .iter()
        .map(|key| *named(&observed, key).as_bytes())
        .collect();
    names.push(*super::outputs::seam_named(&seams, "s").as_bytes());
    names
}

#[test]
fn a_naming_scope_mints_the_names_manual_calls_mint() {
    let feature = occurrence("feature");
    let instance = feature.clone().index("instance", 2).unwrap();
    let manual = World::install();
    manual.committed(
        "scope-manual",
        Plan {
            creates: keys(&["a", "b", "c", "d"]),
            seams: vec![("s".into(), "a".into(), "b".into())],
            steps: vec![
                Step::Birth("a".into(), birth("feature", "authored")),
                Step::BirthSided {
                    key: "b".into(),
                    birth: birth("feature", "piece"),
                    side: CutSide::Negative,
                    of: "a".into(),
                    along: None,
                },
                Step::SeamBirthOf {
                    key: "s".into(),
                    birth: birth("feature", "seam"),
                    index: 1,
                    of: "a".into(),
                },
                Step::Birth(
                    "c".into(),
                    WorthQueryOutputBirth::of_role(&instance, MemberRole::Instance(2)),
                ),
                Step::Birth("d".into(), birth("feature/hole", "authored")),
            ],
            ..Plan::default()
        },
    );
    let scoped = World::install();
    scoped.committed(
        "scope-scoped",
        Plan {
            creates: keys(&["a", "b", "c", "d"]),
            steps: vec![Step::Scoped(
                feature,
                vec![
                    ScopeStep::Name("a".into(), MemberRole::Authored),
                    ScopeStep::CutBy {
                        key: "b".into(),
                        side: CutSide::Negative,
                        of: "a".into(),
                    },
                    ScopeStep::LinkSeamOf {
                        key: "s".into(),
                        from: "a".into(),
                        to: "b".into(),
                        index: 1,
                        of: "a".into(),
                    },
                    ScopeStep::Index(
                        "instance".into(),
                        2,
                        vec![ScopeStep::Name("c".into(), MemberRole::Instance(2))],
                    ),
                    ScopeStep::Child(
                        "hole".into(),
                        vec![ScopeStep::Name("d".into(), MemberRole::Authored)],
                    ),
                ],
            )],
            ..Plan::default()
        },
    );
    let names = authored_names(&manual, "scope-manual-names");
    assert_eq!(authored_names(&scoped, "scope-scoped-names"), names);
    let distinct: std::collections::BTreeSet<_> = names.iter().collect();
    assert_eq!(distinct.len(), names.len(), "every output has its own name");
}

/// A world holding members `x` and `y`, with their names and keys ordered
/// least name first.
fn two_members(token: &str) -> (World, [(WorthQueryName<Member>, &'static str); 2]) {
    let world = World::install();
    world.committed(
        token,
        Plan {
            creates: keys(&["x", "y"]),
            steps: vec![
                Step::Birth("x".into(), birth("least-x", "authored")),
                Step::Birth("y".into(), birth("least-y", "authored")),
            ],
            ..Plan::default()
        },
    );
    let observed = world.probe(&format!("{token}-names"), &["x", "y"], 0);
    let mut held = [(named(&observed, "x"), "x"), (named(&observed, "y"), "y")];
    held.sort();
    (world, held)
}

/// Runs `merge_least` over `x` and `y` listed in `order`, onto created
/// member `t` when `onto`, and returns the world and the ordered parents.
fn merge_least(
    token: &str,
    order: [&str; 2],
    onto: bool,
) -> (World, [(WorthQueryName<Member>, &'static str); 2]) {
    let (world, held) = two_members(token);
    let [_, (_, greater)] = held;
    let deletes = if onto {
        keys(&["x", "y"])
    } else {
        keys(&[greater])
    };
    let creates = if onto { keys(&["t"]) } else { Vec::new() };
    let target = onto.then(|| "t".to_owned());
    world.committed(
        &format!("{token}-merge"),
        Plan {
            reads: keys(&["x", "y"]),
            deletes,
            creates,
            steps: vec![Step::Scoped(
                occurrence("least"),
                vec![ScopeStep::MergeLeast(order.map(read).to_vec(), target)],
            )],
            ..Plan::default()
        },
    );
    (world, held)
}

/// Where the held names `x` and `y` resolve after the merge, among the
/// live members `live`.
fn survivors(
    world: &World,
    token: &str,
    held: &[(WorthQueryName<Member>, &str); 2],
    live: &[&str],
) -> Vec<String> {
    for (name, key) in held {
        probe::hold(token, key, *name);
    }
    let observed = world.probe(token, live, 1);
    held.iter()
        .map(|(_, key)| format!("{:?}", resolution(&observed, key)))
        .collect()
}

#[test]
fn merge_least_keeps_the_least_parent_in_place_in_any_order() {
    let (forward, held) = merge_least("least-in-place-xy", ["x", "y"], false);
    let (backward, again) = merge_least("least-in-place-yx", ["y", "x"], false);
    assert_eq!(held, again, "both worlds mint the same names");
    let [(_, least), (_, greater)] = held;
    for world in [&forward, &backward] {
        assert!(world.live(least) && !world.live(greater));
    }
    let expected: Vec<String> = held
        .iter()
        .map(|(_, label)| format!("{:?}", resolved_to(label, least)))
        .collect();
    for (world, token) in [
        (&forward, "least-in-place-xy-s"),
        (&backward, "least-in-place-yx-s"),
    ] {
        assert_eq!(survivors(world, token, &held, &[least]), expected);
    }
}

#[test]
fn merge_least_carries_the_least_name_onto_a_created_record_in_any_order() {
    let (forward, held) = merge_least("least-onto-xy", ["x", "y"], true);
    let (backward, again) = merge_least("least-onto-yx", ["y", "x"], true);
    assert_eq!(held, again);
    let [(least, _), _] = held;
    for (world, token) in [
        (&forward, "least-onto-xy-t"),
        (&backward, "least-onto-yx-t"),
    ] {
        assert!(world.live("t") && !world.live("x") && !world.live("y"));
        assert_eq!(named(&world.probe(token, &["t"], 0), "t"), least);
    }
    let expected: Vec<String> = held
        .iter()
        .map(|(_, label)| format!("{:?}", resolved_to(label, "t")))
        .collect();
    for (world, token) in [
        (&forward, "least-onto-xy-s"),
        (&backward, "least-onto-yx-s"),
    ] {
        assert_eq!(survivors(world, token, &held, &["t"]), expected);
    }
}

#[test]
fn merge_least_refuses_fewer_than_two_parents_without_effects() {
    let (world, held) = two_members("least-few");
    let [(_, least), (_, greater)] = held;
    let none = world.try_sculpt(
        "least-few-none",
        Plan {
            steps: vec![Step::Scoped(
                occurrence("least"),
                vec![ScopeStep::MergeLeast(Vec::new(), None)],
            )],
            ..Plan::default()
        },
    );
    assert!(
        none.is_err(),
        "no parents refuses while authoring: {none:?}"
    );
    assert_eq!(
        sculpt::authoring_refusal("least-few-none"),
        Some(sculpt::Refusal::EmptyMergeParents)
    );
    let one = world.sculpt(
        "least-few-one",
        Plan {
            reads: keys(&[greater]),
            deletes: keys(&[greater]),
            steps: vec![Step::Scoped(
                occurrence("least"),
                vec![ScopeStep::MergeLeast(vec![read(greater)], None)],
            )],
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            naming_refusal(&one),
            Some(LineageNameDenial::LineagePostureMismatch {
                posture: LineagePostureKind::Merge,
                ..
            })
        ),
        "{one:?}"
    );
    assert!(world.live(least) && world.live(greater), "zero effects");
}

/// Seams `p` (`m1`-`m2`) and `q` (`m1`-`m3`), their names least first.
fn two_seams(token: &str) -> (World, [(WorthQueryName<Seam>, &'static str); 2]) {
    let world = members();
    world.committed(
        &format!("{token}-p"),
        link("p", "m1", "m2", seam("weld-p", "seam", 1)),
    );
    world.committed(
        &format!("{token}-q"),
        link("q", "m1", "m3", seam("weld-q", "seam", 1)),
    );
    let mut held = [
        (
            read_name(&world, &format!("{token}-read-p"), "m1", "m2"),
            "p",
        ),
        (
            read_name(&world, &format!("{token}-read-q"), "m1", "m3"),
            "q",
        ),
    ];
    held.sort();
    (world, held)
}

fn seam_ends(label: &str) -> (&'static str, &'static str) {
    if label == "p" {
        ("m1", "m2")
    } else {
        ("m1", "m3")
    }
}

#[test]
fn merge_least_relation_keeps_the_least_seam_in_any_order() {
    let mut outcomes = Vec::new();
    for (token, order) in [("least-seam-pq", ["p", "q"]), ("least-seam-qp", ["q", "p"])] {
        let (world, held) = two_seams(token);
        let [_, (_, greater)] = held;
        let (from, to) = seam_ends(greater);
        world.committed(
            &format!("{token}-merge"),
            Plan {
                reads: keys(&["m1", "m2", "m3"]),
                seam_reads: vec![seam_read("p", "m1", "m2"), seam_read("q", "m1", "m3")],
                unlinks: vec![(from.into(), to.into())],
                steps: vec![Step::Scoped(
                    occurrence("least-seam"),
                    vec![ScopeStep::SeamMergeLeast(
                        order.map(|label| HeldSeam::Read(label.into())).to_vec(),
                        None,
                    )],
                )],
                ..Plan::default()
            },
        );
        let [(_, least), _] = held;
        let (from, to) = seam_ends(least);
        for (name, label) in held {
            let resolved = resolve(&world, &format!("{token}-{label}"), name, exact()).0;
            assert_eq!(resolved, seam_between("held", from, to));
        }
        outcomes.push((held, ends(from, to)));
    }
    assert_eq!(outcomes[0], outcomes[1]);
}
