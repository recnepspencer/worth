//! Installed-operation births, in-commit regeneration, splits and merges.
use super::*;
use probe::Observation;
use worth_query_decl::facade::application_naming::NameSelection;

#[test]
fn independent_occurrences_stay_distinct_and_regeneration_preserves_the_name() {
    let world = World::install();
    world.committed(
        "occurrences",
        Plan {
            creates: keys(&["f1", "f2"]),
            steps: vec![
                Step::Birth("f1".into(), birth("feature-1", "authored")),
                Step::Birth("f2".into(), birth("feature-2", "authored")),
            ],
            ..Plan::default()
        },
    );
    let observed = world.probe("occurrences", &["f1", "f2"], 0);
    let (f1, f2) = (named(&observed, "f1"), named(&observed, "f2"));
    assert_ne!(
        f1, f2,
        "identical payloads of two authored occurrences mint distinct names"
    );

    // Retiring the holder and re-minting the same occurrence and role in one
    // operation is `Preserve` continuity: the regenerated record keeps `f1`.
    world.committed(
        "regenerate",
        Plan {
            reads: keys(&["f1"]),
            deletes: keys(&["f1"]),
            creates: keys(&["f1r"]),
            steps: vec![
                Step::Retire(read("f1")),
                Step::Birth("f1r".into(), birth("feature-1", "authored")),
            ],
            ..Plan::default()
        },
    );
    probe::hold("regenerate", "f1", f1);
    let observed = world.probe("regenerate", &["f1r"], 1);
    assert_eq!(
        named(&observed, "f1r"),
        f1,
        "regeneration keeps the exact name"
    );
    assert_eq!(resolution(&observed, "f1"), &resolved_to("f1", "f1r"));

    // Re-minting a name whose holder stays live refuses with zero effects.
    let outcome = world.sculpt(
        "remint",
        Plan {
            creates: keys(&["dup"]),
            steps: vec![Step::Birth("dup".into(), birth("feature-2", "authored"))],
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            naming_refusal(&outcome),
            Some(LineageNameDenial::DuplicateBirth { .. })
        ),
        "{outcome:?}"
    );
    assert!(!world.live("dup"), "a refused re-mint leaves no record");
    assert_eq!(named(&world.probe("remint", &["f2"], 2), "f2"), f2);
}

#[test]
fn split_pieces_are_ambiguous_exactly_and_plural_over_descendants_then_merge() {
    let world = World::install();
    world.committed(
        "split",
        Plan {
            creates: keys(&["body", "tool"]),
            steps: vec![
                Step::Birth("body".into(), birth("body", "authored")),
                Step::Birth("tool".into(), birth("tool", "authored")),
            ],
            ..Plan::default()
        },
    );
    let observed = world.probe("split", &["body", "tool"], 0);
    let (body, tool) = (named(&observed, "body"), named(&observed, "tool"));

    // Split `body` into two pieces told apart by side and by the cutter's name.
    world.committed(
        "split-body",
        Plan {
            reads: keys(&["body"]),
            deletes: keys(&["body"]),
            creates: keys(&["left", "right"]),
            steps: vec![Step::Split(
                read("body"),
                vec![
                    ("left".into(), piece("cut-1", 1, Some(tool))),
                    ("right".into(), piece("cut-1", 2, Some(tool))),
                ],
            )],
            ..Plan::default()
        },
    );
    let observed = world.probe("split-body", &["left", "right"], 1);
    let (left, right) = (named(&observed, "left"), named(&observed, "right"));
    assert!(left != right && left != body && right != body);
    probe::hold("split-body", "exact", body);
    probe::hold_selecting("split-body", "all", body, NameSelection::AllDescendants);
    let observed = world.probe("split-body", &["left", "right"], 2);
    let mut expected = vec![left, right];
    expected.sort();
    assert_eq!(
        resolution(&observed, "exact"),
        &Observation::Ambiguous {
            label: "exact".into(),
            candidates: expected
        },
        "Exact never picks a least member"
    );
    match resolution(&observed, "all") {
        Observation::Plural { keys, .. } => {
            let mut keys = keys.clone();
            keys.sort();
            assert_eq!(keys, ["left", "right"]);
        }
        other => panic!("AllDescendants returns both live pieces: {other:?}"),
    }

    // Merge keeping `left` in place: `right`'s held name follows to `left`.
    world.committed(
        "merge-keep",
        Plan {
            reads: keys(&["left", "right"]),
            deletes: keys(&["right"]),
            steps: vec![Step::Merge(
                vec![read("left"), read("right")],
                Keeper::InPlace(read("left")),
            )],
            ..Plan::default()
        },
    );
    probe::hold("merge-keep", "right", right);
    probe::hold("merge-keep", "body", body);
    let observed = world.probe("merge-keep", &["left"], 3);
    assert_eq!(
        named(&observed, "left"),
        left,
        "the kept parent keeps its name"
    );
    assert_eq!(
        resolution(&observed, "right"),
        &resolved_to("right", "left")
    );
    assert_eq!(resolution(&observed, "body"), &resolved_to("body", "left"));
}

#[test]
fn merge_minting_a_new_name_resolves_every_parent_to_the_survivor() {
    let world = World::install();
    world.committed(
        "mint",
        Plan {
            creates: keys(&["q1", "q2"]),
            steps: vec![
                Step::Birth("q1".into(), birth("q-1", "authored")),
                Step::Birth("q2".into(), birth("q-2", "authored")),
            ],
            ..Plan::default()
        },
    );
    let observed = world.probe("mint", &["q1", "q2"], 0);
    let (q1, q2) = (named(&observed, "q1"), named(&observed, "q2"));
    world.committed(
        "mint-merge",
        Plan {
            reads: keys(&["q1", "q2"]),
            deletes: keys(&["q1", "q2"]),
            creates: keys(&["merged"]),
            steps: vec![Step::Merge(
                vec![read("q1"), read("q2")],
                Keeper::MintNew(
                    "merged".into(),
                    birth("merge-1", "merged").with_discriminator(&MemberCut::Side {
                        side: 0,
                        cutter: None,
                    }),
                ),
            )],
            ..Plan::default()
        },
    );
    probe::hold("mint-merge", "q1", q1);
    probe::hold("mint-merge", "q2", q2);
    let observed = world.probe("mint-merge", &["merged"], 1);
    let merged = named(&observed, "merged");
    assert!(
        merged != q1 && merged != q2,
        "MintNew mints a distinct name"
    );
    assert_eq!(resolution(&observed, "q1"), &resolved_to("q1", "merged"));
    assert_eq!(resolution(&observed, "q2"), &resolved_to("q2", "merged"));
}
