//! Crash and reopen at every lineage write boundary (Phase 7, Arch 8, 22).
//!
//! Two journeys on a real Directory Home, each commit followed by a crash
//! and a reopen as a fresh installation, alternately from a checkpoint
//! captured at that boundary and by replaying the Journal since the last
//! one: entity, seam (relation) and group `Create`, a seam `Merge`
//! keeping the least, a multi-body `Split` with regrouping, a
//! `KeepParentInPlace` merge; then `MintNew`, in-commit regeneration
//! (`Preserve`), `Retire` and cross-commit `Restore`. Every retained name
//! must be served identically before and after each reopen.
use super::super::groups::loop_of;
use super::super::outputs::sculpt::{
    GroupDecl, GroupMember, Held, HeldSeam, Keeper, Partition, Plan, ScopeStep, Step,
};
use super::super::outputs::{birth, keys, read, CutSide, MemberCut, SidedCut, World};
use super::super::world::home;
use super::*;
use worth_query_decl::facade::application_naming::{NameRef, OccurrencePath};
use worth_query_host::facade::application_naming::WorthQueryOutputBirth;
use GroupRole::{Host, Member as Plain, Producer};

fn occurrence(path: &str) -> OccurrencePath {
    OccurrencePath::root(path).expect("a valid occurrence")
}

fn created(key: &str) -> GroupMember {
    GroupMember::Created(key.to_owned())
}

/// Faces `f1`, `f2` (cut by `f1`), two parallel seams, a loop and a shell.
fn faces_and_seams() -> Plan {
    let seam = |key: &str, from: &str, to: &str, index| ScopeStep::LinkSeamOf {
        key: key.into(),
        from: from.into(),
        to: to.into(),
        index,
        of: "dj-f1".into(),
    };
    fn partition<G>(first: GroupRole, second: GroupRole) -> Partition<G> {
        Partition {
            groups: vec![vec![(first, created("dj-f1")), (second, created("dj-f2"))]],
            dissolve: Vec::new(),
        }
    }
    Plan {
        creates: keys(&["dj-f1", "dj-f2", "dj-f3"]),
        steps: vec![
            Step::Scoped(
                occurrence("durable-body-a"),
                vec![
                    ScopeStep::Name("dj-f1".into(), MemberRole::Authored),
                    ScopeStep::CutBy {
                        key: "dj-f2".into(),
                        side: CutSide::Positive,
                        of: "dj-f1".into(),
                    },
                    seam("dj-s0", "dj-f1", "dj-f2", 0),
                    seam("dj-s1", "dj-f2", "dj-f1", 1),
                ],
            ),
            Step::Birth("dj-f3".into(), birth("dj-f3", "authored")),
            Step::Groups(GroupDecl::Loop(partition(Host, Plain))),
            Step::Groups(GroupDecl::Shell(partition(Producer, Plain))),
        ],
        ..Plan::default()
    }
}

fn piece(cutter: WorthQueryName<Member>, side: CutSide) -> WorthQueryOutputBirth {
    let cut = SidedCut::By {
        cutter: NameRef::Committed(cutter),
        side,
        along: None,
    };
    WorthQueryOutputBirth::of_role(&occurrence("durable-split"), MemberRole::Piece)
        .with_discriminator(&cut)
}

#[test]
fn a_cell_complex_survives_reopen_after_every_commit() {
    let mut world = World::open_directory(home::location());
    let mut held = Holdings::default();
    world.committed("dj-faces", faces_and_seams());
    let names = world.names("dj-face-names", &["dj-f1", "dj-f2", "dj-f3"], 0);
    let (f1, f2, f3) = (names[0], names[1], names[2]);
    let pairs = vec![
        ("s0".into(), "dj-f1".into(), "dj-f2".into()),
        ("s1".into(), "dj-f2".into(), "dj-f1".into()),
    ];
    let _ = world.try_sculpt(
        "dj-seam-names",
        Plan {
            reads: keys(&["dj-f1", "dj-f2"]),
            seam_reads: pairs.clone(),
            ..Plan::default()
        },
    );
    let seams = super::super::outputs::sculpt::seam_names("dj-seam-names");
    // Groups are committed and their lineage persisted here; the held group
    // reads across reopen are `groups_reopen`'s, one checkpoint form held.
    assert!(
        loop_of(&world, "dj-loop", f1).is_some(),
        "the loop is born with its faces"
    );
    held.keys = ["dj-f1", "dj-f2", "dj-f3"].map(String::from).to_vec();
    held.member("f1", f1);
    held.member("f2", f2);
    held.member("f3", f3);
    held.seams = vec![("s0".into(), seams["s0"]), ("s1".into(), seams["s1"])];
    held.seam_pairs = pairs.clone();
    crash_and_compare(&mut world, &held, "create", false);

    let away = if seams["s0"] < seams["s1"] {
        ("dj-f2", "dj-f1")
    } else {
        ("dj-f1", "dj-f2")
    };
    world.committed(
        "dj-seam-merge",
        Plan {
            reads: keys(&["dj-f1", "dj-f2"]),
            seam_reads: pairs,
            unlinks: vec![(away.0.into(), away.1.into())],
            steps: vec![Step::Scoped(
                occurrence("durable-seams"),
                vec![ScopeStep::SeamMergeLeast(
                    vec![HeldSeam::Read("s0".into()), HeldSeam::Read("s1".into())],
                    None,
                )],
            )],
            ..Plan::default()
        },
    );
    crash_and_compare(&mut world, &held, "seam-merge", true);

    world.committed(
        "dj-split",
        Plan {
            reads: keys(&["dj-f2"]),
            deletes: keys(&["dj-f2"]),
            creates: keys(&["dj-f2a", "dj-f2b"]),
            owners: vec![("dj-f2b".into(), "b".into())],
            steps: vec![
                Step::Split(
                    Held::Read("dj-f2".into()),
                    vec![
                        ("dj-f2a".into(), piece(f1, CutSide::Positive)),
                        ("dj-f2b".into(), piece(f1, CutSide::Negative)),
                    ],
                ),
                Step::Groups(GroupDecl::Loop(Partition {
                    groups: vec![vec![(Host, created("dj-f2b"))]],
                    dissolve: Vec::new(),
                })),
            ],
            ..Plan::default()
        },
    );
    let pieces = world.names("dj-piece-names", &["dj-f2a", "dj-f2b"], 1);
    held.keys.extend(["dj-f2a", "dj-f2b"].map(String::from));
    held.member("f2a", pieces[0]);
    held.member("f2b", pieces[1]);
    crash_and_compare(&mut world, &held, "split", false);

    world.committed(
        "dj-keep-in-place",
        Plan {
            reads: keys(&["dj-f3"]),
            deletes: keys(&["dj-f3"]),
            steps: vec![Step::Merge(
                vec![Held::Name(pieces[0]), read("dj-f3")],
                Keeper::InPlace(Held::Name(pieces[0])),
            )],
            ..Plan::default()
        },
    );
    crash_and_compare(&mut world, &held, "merge-keep-in-place", true);
}

fn merged() -> WorthQueryOutputBirth {
    let cut = MemberCut::Side {
        side: 0,
        cutter: None,
    };
    birth("durable-merge", "merged").with_discriminator(&cut)
}

#[test]
fn minted_regenerated_retired_and_restored_names_survive_reopen() {
    let mut world = World::open_directory(home::location());
    let mut held = Holdings::default();
    world.committed(
        "dr-create",
        Plan {
            creates: keys(&["dr-g1", "dr-g2", "dr-g6"]),
            steps: vec![
                Step::Birth("dr-g1".into(), birth("dr-g1", "authored")),
                Step::Birth("dr-g2".into(), birth("dr-g2", "authored")),
                Step::Birth("dr-g6".into(), birth("dr-g6", "authored")),
            ],
            ..Plan::default()
        },
    );
    let parents = world.names("dr-parents", &["dr-g1", "dr-g2", "dr-g6"], 0);
    held.keys = ["dr-g1", "dr-g2", "dr-g3", "dr-g4", "dr-g5", "dr-g6"]
        .map(String::from)
        .to_vec();
    held.member("g1", parents[0]);
    held.member("g2", parents[1]);
    held.member("g6", parents[2]);
    let authored = parents[2];
    crash_and_compare(&mut world, &held, "create", true);

    world.committed(
        "dr-mint",
        Plan {
            reads: keys(&["dr-g1", "dr-g2"]),
            deletes: keys(&["dr-g1", "dr-g2"]),
            creates: keys(&["dr-g3"]),
            steps: vec![Step::Merge(
                vec![read("dr-g1"), read("dr-g2")],
                Keeper::MintNew("dr-g3".into(), merged()),
            )],
            ..Plan::default()
        },
    );
    let minted = world.names("dr-minted", &["dr-g3"], 1)[0];
    held.member("g3", minted);
    crash_and_compare(&mut world, &held, "merge-mint", false);

    // In-commit regeneration: the same birth re-minted onto a fresh record.
    // Replayed from the Journal alone (no checkpoint at this boundary).
    world.committed(
        "dr-preserve",
        Plan {
            reads: keys(&["dr-g6"]),
            deletes: keys(&["dr-g6"]),
            creates: keys(&["dr-g4"]),
            steps: vec![
                Step::Retire(read("dr-g6")),
                Step::Birth("dr-g4".into(), birth("dr-g6", "authored")),
            ],
            ..Plan::default()
        },
    );
    assert_eq!(world.names("dr-preserved", &["dr-g4"], 2), vec![authored]);
    crash_and_compare(&mut world, &held, "preserve", false);

    world.committed(
        "dr-retire",
        Plan {
            reads: keys(&["dr-g4"]),
            deletes: keys(&["dr-g4"]),
            steps: vec![Step::Retire(read("dr-g4"))],
            ..Plan::default()
        },
    );
    crash_and_compare(&mut world, &held, "retire", true);

    world.committed(
        "dr-restore",
        Plan {
            creates: keys(&["dr-g5"]),
            steps: vec![Step::Birth("dr-g5".into(), birth("dr-g6", "authored"))],
            ..Plan::default()
        },
    );
    assert_eq!(world.names("dr-restored", &["dr-g5"], 3), vec![authored]);
    crash_and_compare(&mut world, &held, "restore", false);
}
