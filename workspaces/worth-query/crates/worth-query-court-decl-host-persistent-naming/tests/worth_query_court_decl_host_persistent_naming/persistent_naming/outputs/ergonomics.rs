//! Phase 5E slice 1: naming declared as ordinary Rust types. The schemes take
//! their roles from `#[derive(WorthQueryNameRole)]` enums, a birth names its
//! role by value in a structured `OccurrencePath`, and one
//! `#[derive(ApplicationNameDiscriminator)]` over `NameRef` serves a cutter
//! born earlier and a cutter born by the same commit: the piece's name is
//! byte-identical either way.
use super::*;
use worth_query_decl::facade::application_naming::{
    IndexedRoleDiscriminator, OccurrencePath, WorthQueryNameRole,
};

/// The hole of repetition `instance` of one authored pattern.
fn hole(instance: u32) -> OccurrencePath {
    OccurrencePath::root("pattern_1")
        .and_then(|path| path.index("instance", instance))
        .and_then(|path| path.child("hole"))
        .expect("a well-formed occurrence path")
}

fn born(occurrence: &OccurrencePath, role: MemberRole) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::of_role(occurrence, role)
}

/// The tool `c` and the seam `s` from `a` to it, authored beside `a`.
fn seam_to_tool(occurrence: &OccurrencePath) -> Vec<Step> {
    vec![
        Step::Birth("c".into(), birth("tool", "authored")),
        Step::SeamBirthOf {
            key: "s".into(),
            birth: WorthQueryOutputBirth::of_role(occurrence, SeamRole::Seam),
            index: 1,
            of: "a".into(),
        },
    ]
}

/// The name of `key`, read by a later operation's decision.
fn name_of(world: &World, token: &str, key: &str) -> WorthQueryName<Member> {
    named(&world.probe(token, &[key], 0), key)
}

#[test]
fn a_discriminator_declared_once_names_a_piece_the_same_in_one_commit_or_two() {
    let occurrence = hole(10);
    assert_eq!(
        occurrence.to_occurrence_key(),
        "pattern_1/instance[10]/hole"
    );

    // One commit: the authored member, the seam beside it and the piece cut
    // by the member along the seam, both referenced by same-commit names.
    let one = World::install();
    let mut steps = vec![Step::Birth(
        "a".into(),
        born(&occurrence, MemberRole::Authored),
    )];
    steps.extend(seam_to_tool(&occurrence));
    steps.push(Step::BirthSided {
        key: "b".into(),
        birth: born(&occurrence, MemberRole::Piece),
        side: CutSide::Positive,
        of: "a".into(),
        along: Some("s".into()),
    });
    one.committed(
        "ergonomics-one",
        Plan {
            creates: keys(&["a", "c", "b"]),
            seams: vec![("s".into(), "a".into(), "c".into())],
            steps,
            ..Plan::default()
        },
    );
    let (a, b) = (
        name_of(&one, "ergonomics-one-a", "a"),
        name_of(&one, "ergonomics-one-b", "b"),
    );
    assert_ne!(a, b);

    // Two commits: the member and the seam are born first, and the piece's
    // discriminator, the same type, holds their committed names.
    let two = World::install();
    let mut steps = vec![Step::Birth(
        "a".into(),
        born(&occurrence, MemberRole::Authored),
    )];
    steps.extend(seam_to_tool(&occurrence));
    two.committed(
        "ergonomics-two-a",
        Plan {
            creates: keys(&["a", "c"]),
            seams: vec![("s".into(), "a".into(), "c".into())],
            steps,
            ..Plan::default()
        },
    );
    probe::seams::name("ergonomics-two-held", "s", "a", "c");
    let held_a = named(&two.probe("ergonomics-two-held", &["a", "c"], 0), "a");
    let held_s = seam_named(&probe::seams::take("ergonomics-two-held"), "s");
    assert_eq!(held_a, a);
    let cut = |side| SidedCut::By {
        cutter: held_a.into(),
        side,
        along: Some(held_s.into()),
    };
    two.committed(
        "ergonomics-two-b",
        Plan {
            creates: keys(&["b", "n"]),
            steps: vec![
                Step::Birth(
                    "b".into(),
                    born(&occurrence, MemberRole::Piece)
                        .with_discriminator(&cut(CutSide::Positive)),
                ),
                Step::Birth(
                    "n".into(),
                    born(&occurrence, MemberRole::Piece)
                        .with_discriminator(&cut(CutSide::Negative)),
                ),
            ],
            ..Plan::default()
        },
    );
    assert_eq!(
        name_of(&two, "ergonomics-two-b-name", "b"),
        b,
        "one declaration, two routes, one name"
    );
    // The nested side is part of the name.
    assert_ne!(name_of(&two, "ergonomics-two-n-name", "n"), b);

    // The typed role and path are the birth a role name and key spell out.
    let spelled = World::install();
    spelled.committed(
        "ergonomics-spelled",
        Plan {
            creates: keys(&["a"]),
            steps: vec![Step::Birth(
                "a".into(),
                birth("pattern_1/instance[10]/hole", "authored"),
            )],
            ..Plan::default()
        },
    );
    assert_eq!(name_of(&spelled, "ergonomics-spelled-a", "a"), a);
}

/// One authored member and three repetitions of the `instance` role in one
/// occurrence; the tenth is also told apart by the member, born beside it.
fn instances(creates: &[&str], occurrence: &OccurrencePath) -> Plan {
    Plan {
        creates: keys(creates),
        steps: vec![
            Step::Birth("a".into(), born(occurrence, MemberRole::Authored)),
            Step::Birth("i2".into(), born(occurrence, MemberRole::Instance(2))),
            Step::Birth("i9".into(), born(occurrence, MemberRole::Instance(9))),
            Step::BirthSided {
                key: "i10".into(),
                birth: born(occurrence, MemberRole::Instance(10)),
                side: CutSide::Positive,
                of: "a".into(),
                along: None,
            },
        ],
        ..Plan::default()
    }
}

fn instance_names(world: &World, token: &str) -> [WorthQueryName<Member>; 3] {
    let observed = world.probe(token, &["i2", "i9", "i10"], 0);
    ["i2", "i9", "i10"].map(|key| named(&observed, key))
}

#[test]
fn indexed_roles_commit_under_their_declared_role_and_order_by_number() {
    // An index never becomes part of the role name Relational checks: every
    // repetition is born under the one declared role.
    for index in [2, 9, 10] {
        let role = MemberRole::Instance(index);
        assert_eq!(
            (role.role_name(), role.role_index()),
            ("instance", Some(index))
        );
        assert!(MemberRole::ROLE_ORDER.contains(&role.role_name()));
        assert_eq!(born(&hole(1), role).role_path(), "instance");
    }
    assert!(MemberRole::Instance(10) > MemberRole::Instance(9));
    assert!(MemberRole::Instance(9) > MemberRole::Instance(2));
    let mut roles = [
        MemberRole::Instance(10),
        MemberRole::Merged,
        MemberRole::Instance(2),
        MemberRole::Piece,
        MemberRole::Instance(9),
        MemberRole::Seam,
        MemberRole::Authored,
    ];
    roles.sort();
    let declared: Vec<_> = roles
        .iter()
        .map(|role| (role.role_name(), role.role_index()))
        .collect();
    assert_eq!(
        declared,
        [
            ("authored", None),
            ("seam", None),
            ("piece", None),
            ("merged", None),
            ("instance", Some(2)),
            ("instance", Some(9)),
            ("instance", Some(10)),
        ]
    );
    assert_eq!(MemberRole::ROLE_ORDER, roles_in_order(&roles));
    assert!(SeamRole::Seam > SeamRole::Authored && SeamRole::SeamPiece > SeamRole::Seam);
    assert!(hole(10) > hole(9) && hole(9) > hole(2));

    // The repetitions commit, and each mints its own name.
    let world = World::install();
    world.committed("instances", instances(&["a", "i2", "i9", "i10"], &hole(1)));
    let [i2, i9, i10] = instance_names(&world, "instance-names");
    assert!(i2 != i9 && i9 != i10 && i2 != i10);

    // Created in another order, the same repetitions mint the same names.
    let rebuilt = World::install();
    rebuilt.committed(
        "instances-rebuilt",
        instances(&["i10", "i9", "a", "i2"], &hole(1)),
    );
    assert_eq!(
        instance_names(&rebuilt, "instance-names-rebuilt"),
        [i2, i9, i10]
    );

    // The tenth repetition born by a later commit, its discriminator holding
    // the member's committed name, is the same name again.
    let staged = World::install();
    staged.committed(
        "instances-staged-a",
        Plan {
            creates: keys(&["a"]),
            steps: vec![Step::Birth(
                "a".into(),
                born(&hole(1), MemberRole::Authored),
            )],
            ..Plan::default()
        },
    );
    let cut = SidedCut::By {
        cutter: name_of(&staged, "instances-staged-held", "a").into(),
        side: CutSide::Positive,
        along: None,
    };
    staged.committed(
        "instances-staged-i10",
        Plan {
            creates: keys(&["i10"]),
            steps: vec![Step::Birth(
                "i10".into(),
                born(&hole(1), MemberRole::Instance(10)).with_discriminator(&cut),
            )],
            ..Plan::default()
        },
    );
    assert_eq!(name_of(&staged, "instances-staged-name", "i10"), i10);

    // The same repetitions of another indexed occurrence are other names.
    let other = World::install();
    other.committed(
        "instances-other",
        instances(&["a", "i2", "i9", "i10"], &hole(10)),
    );
    let moved = instance_names(&other, "instance-names-other");
    assert!(moved[0] != i2 && moved[1] != i9 && moved[2] != i10);
}

#[test]
fn a_split_piece_born_under_an_indexed_role_is_kept_by_its_index_and_side() {
    let world = World::install();
    world.committed(
        "indexed-split",
        Plan {
            creates: keys(&["body"]),
            steps: vec![Step::Birth(
                "body".into(),
                born(&hole(1), MemberRole::Authored),
            )],
            ..Plan::default()
        },
    );
    let body = name_of(&world, "indexed-split-body", "body");

    // Three repetitions split from `body`: two on the same side, told apart
    // by index alone, and one with no side of its own.
    let side = || MemberCut::Side {
        side: 1,
        cutter: None,
    };
    let piece = |index| born(&hole(1), MemberRole::Instance(index));
    world.committed(
        "indexed-split-cut",
        Plan {
            reads: keys(&["body"]),
            deletes: keys(&["body"]),
            creates: keys(&["first", "second", "third"]),
            steps: vec![Step::Split(
                read("body"),
                vec![
                    ("first".into(), piece(1).with_discriminator(&side())),
                    ("second".into(), piece(2).with_discriminator(&side())),
                    ("third".into(), piece(3)),
                ],
            )],
            ..Plan::default()
        },
    );

    let keep = |label: &str, selection| {
        probe::hold_selecting("indexed-split-kept", label, body, selection);
    };
    let indexed = |index| NameSelection::keep_side(&IndexedRoleDiscriminator::new(index, side()));
    keep("second", indexed(2).unwrap());
    keep("first", indexed(1).unwrap());
    keep(
        "third",
        NameSelection::keep_side(&IndexedRoleDiscriminator::index_alone(3)).unwrap(),
    );
    // The side without its index, and an index no piece was born under,
    // match nothing and refuse rather than pick a piece.
    keep("unindexed", NameSelection::keep_side(&side()).unwrap());
    keep("absent", indexed(4).unwrap());
    let observed = world.probe("indexed-split-kept", &["first", "second", "third"], 0);
    for key in ["first", "second", "third"] {
        assert_eq!(resolution(&observed, key), &resolved_to(key, key));
    }
    for label in ["unindexed", "absent"] {
        assert!(
            matches!(
                resolution(&observed, label),
                probe::Observation::ResolveRefused { refusal, .. }
                    if refusal.contains("NoMatchingCandidate")
            ),
            "{label}: {observed:?}"
        );
    }
}

/// The declared role names of sorted `roles`, without repeats.
fn roles_in_order(roles: &[MemberRole]) -> Vec<&'static str> {
    let mut names: Vec<_> = roles.iter().map(MemberRole::role_name).collect();
    names.dedup();
    names
}
