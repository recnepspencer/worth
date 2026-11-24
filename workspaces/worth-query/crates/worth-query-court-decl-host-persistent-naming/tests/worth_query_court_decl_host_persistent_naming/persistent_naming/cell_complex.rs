//! The Phase 5E hard DX bar: one CAD-shaped cell complex authored exactly as
//! an application would, across the whole persistent-naming stack.
//!
//! Bodies are models `a` and `b`; faces are members owned through their
//! body's `ModelMember` chain; seams are parallel `Seam` relations between
//! faces; a face loop (`Loop`, `Anchor` continuity) and a shell (`Shell`,
//! `ProducerRole` continuity) group faces. One two-stage edit envelope builds
//! the complex: its first stage names faces and two parallel seams through a
//! naming scope and groups the faces by their same-commit names; its second
//! stage reads a held face's name on the envelope's candidate and adds a
//! face. A later commit splits a face into pieces owned by two bodies and
//! regroups them, and a final one merges faces and seams keeping the least.
//!
//! Every role is a `WorthQueryNameRole` value, every discriminator a derived
//! or declared typed value, every reference a `NameRef` or `WorthQueryName`
//! stored through the stock bindings: no role text, byte encoder or codec.
use super::groups::{read, the_loop};
use super::named_read::{self, GroupedMember};
use super::outputs::sculpt::{
    GroupAnswer, GroupDecl, GroupMember, GroupName, GroupRead, Held, HeldSeam, Partition, Plan,
    ScopeStep, Step,
};
use super::outputs::{exact, keys, named, resolution, resolved_to, CutSide, SidedCut, World};
use super::*;
use probe::Observation;
use worth_query_decl::facade::application_naming::{
    NameRef, NameSelection, NamedTarget, OccurrencePath, WorthQueryName,
};
use worth_query_host::facade::application_naming::WorthQueryOutputBirth;
use GroupRole::{Host, Member as Plain, Producer};

fn occurrence(path: &str) -> OccurrencePath {
    OccurrencePath::root(path).expect("a valid occurrence")
}

fn created(key: &str) -> GroupMember {
    GroupMember::Created(key.to_owned())
}

fn held(name: WorthQueryName<Member>) -> GroupMember {
    GroupMember::Held(Held::Name(name))
}

/// A face piece cut from `cutter` on `side`, born under the typed role.
fn piece(at: &str, cutter: WorthQueryName<Member>, side: CutSide) -> WorthQueryOutputBirth {
    let cut = SidedCut::By {
        cutter: NameRef::Committed(cutter),
        side,
        along: None,
    };
    WorthQueryOutputBirth::of_role(&occurrence(at), MemberRole::Piece).with_discriminator(&cut)
}

fn rows(world: &World, keys: &[&str]) -> Vec<GroupedMember> {
    let request = world.request();
    let on_world = request.on_branch(world.host.current_world());
    keys.iter()
        .flat_map(|key| {
            let read = on_world.query(named_read::grouped_member::Read((*key).into()));
            read.execute()
                .unwrap_or_else(|cause| panic!("{key}: {cause:?}"))
                .rows()
                .to_vec()
        })
        .collect()
}

/// Stage one: two faces and two parallel seams through one naming scope,
/// then a loop and a shell over the faces by their same-commit names.
fn faces_and_seams() -> Plan {
    let loops = GroupDecl::Loop(Partition {
        groups: vec![vec![(Host, created("cx-f1")), (Plain, created("cx-f2"))]],
        dissolve: Vec::new(),
    });
    let shells = GroupDecl::Shell(Partition {
        groups: vec![vec![
            (Producer, created("cx-f1")),
            (Plain, created("cx-f2")),
        ]],
        dissolve: Vec::new(),
    });
    let seam = |key: &str, from: &str, to: &str, index| ScopeStep::LinkSeamOf {
        key: key.into(),
        from: from.into(),
        to: to.into(),
        index,
        of: "cx-f1".into(),
    };
    Plan {
        creates: keys(&["cx-f1", "cx-f2"]),
        steps: vec![
            Step::Scoped(
                occurrence("cell-body-a"),
                vec![
                    ScopeStep::Name("cx-f1".into(), MemberRole::Authored),
                    ScopeStep::CutBy {
                        key: "cx-f2".into(),
                        side: CutSide::Positive,
                        of: "cx-f1".into(),
                    },
                    seam("cx-s0", "cx-f1", "cx-f2", 0),
                    seam("cx-s1", "cx-f2", "cx-f1", 1),
                ],
            ),
            Step::Groups(loops),
            Step::Groups(shells),
        ],
        ..Plan::default()
    }
}

/// Stage two reads a seeded face on the envelope's candidate and adds one.
fn candidate_stage() -> Plan {
    Plan {
        reads: keys(&["direct"]),
        creates: keys(&["cx-f3"]),
        steps: vec![Step::Scoped(
            occurrence("cell-body-a-stage-2"),
            vec![ScopeStep::Name("cx-f3".into(), MemberRole::Authored)],
        )],
        ..Plan::default()
    }
}

fn loop_of(world: &World, token: &str, face: WorthQueryName<Member>) -> Option<GroupName> {
    match read(world, token, vec![GroupRead::LoopOf(Held::Name(face))]).as_slice() {
        [GroupAnswer::Of(group)] => *group,
        other => panic!("{token}: {other:?}"),
    }
}

#[test]
fn a_cell_complex_names_bodies_faces_seams_and_groups_end_to_end() {
    let world = World::install();
    world.committed_envelope(
        "cell-envelope",
        vec![
            super::coupled::stage("cell-stage-1", faces_and_seams()),
            super::coupled::stage("cell-stage-2", candidate_stage()),
        ],
    );
    // Projected reads: each face's own name and its groups, in one read.
    let built = rows(&world, &["cx-f1", "cx-f2", "cx-f3"]);
    let (f1, f2) = (built[0].name, built[1].name);
    let face_loop = built[0].in_loop.expect("the loop is born with its faces");
    let shell = built[0].in_shell.expect("the shell is born with its faces");
    assert_eq!(
        (built[1].in_loop, built[1].in_shell),
        (Some(face_loop), Some(shell))
    );
    assert_eq!((built[2].in_loop, built[2].in_shell), (None, None));
    assert_eq!(
        world.names("cell-names", &["cx-f1", "cx-f2"], 0),
        vec![f1, f2]
    );

    // A body holds a face by persistent name in its own record.
    world.hold("cell-hold", NamedTarget::exact(f2));

    // Merge the two parallel seams keeping the least name in place.
    let seam_reads = vec![
        ("s0".into(), "cx-f1".into(), "cx-f2".into()),
        ("s1".into(), "cx-f2".into(), "cx-f1".into()),
    ];
    let read_seams = Plan {
        reads: keys(&["cx-f1", "cx-f2"]),
        seam_reads: seam_reads.clone(),
        ..Plan::default()
    };
    let _ = world.try_sculpt("cell-seam-names", read_seams);
    let seams = super::outputs::sculpt::seam_names("cell-seam-names");
    let away = if seams["s0"] < seams["s1"] {
        ("cx-f2", "cx-f1")
    } else {
        ("cx-f1", "cx-f2")
    };
    world.committed(
        "cell-seam-merge",
        Plan {
            reads: keys(&["cx-f1", "cx-f2"]),
            seam_reads,
            unlinks: vec![(away.0.into(), away.1.into())],
            steps: vec![Step::Scoped(
                occurrence("cell-seams"),
                vec![ScopeStep::SeamMergeLeast(
                    vec![HeldSeam::Read("s0".into()), HeldSeam::Read("s1".into())],
                    None,
                )],
            )],
            ..Plan::default()
        },
    );

    // Split `f2` into pieces owned by two bodies, and regroup: the loop
    // keeps its host's piece; the other piece starts its own loop.
    let split = Plan {
        reads: keys(&["cx-f2"]),
        deletes: keys(&["cx-f2"]),
        creates: keys(&["cx-f2a", "cx-f2b"]),
        owners: vec![("cx-f2b".into(), "b".into())],
        steps: vec![
            Step::Split(
                Held::Read("cx-f2".into()),
                vec![
                    ("cx-f2a".into(), piece("cell-split", f1, CutSide::Positive)),
                    ("cx-f2b".into(), piece("cell-split", f1, CutSide::Negative)),
                ],
            ),
            Step::Groups(GroupDecl::Loop(Partition {
                groups: vec![vec![(Host, created("cx-f2b"))]],
                dissolve: Vec::new(),
            })),
            Step::Groups(GroupDecl::Shell(Partition {
                groups: vec![vec![(Producer, held(f1)), (Plain, created("cx-f2a"))]],
                dissolve: Vec::new(),
            })),
        ],
        ..Plan::default()
    };
    world.committed("cell-split", split);
    let after = rows(&world, &["cx-f1", "cx-f2a", "cx-f2b"]);
    assert_eq!(after[0].in_loop, Some(face_loop), "the host keeps the loop");
    assert_eq!(
        after[0].in_shell,
        Some(shell),
        "the producer keeps the shell"
    );
    assert_eq!(after[1].in_shell, Some(shell));
    let piece_loop = after[2]
        .in_loop
        .expect("the foreign piece has its own loop");
    assert_ne!(piece_loop, face_loop);

    // The split face's name: ambiguous across bodies, exact within each.
    let token = "cell-owned";
    probe::hold_owned(token, "anywhere", f2, exact());
    probe::hold_in(token, "in-a", f2, exact(), "a");
    probe::hold_in(token, "in-b", f2, exact(), "b");
    let observed = world.probe(token, &["cx-f2a", "cx-f2b", "cx-f3"], 1);
    let (f2b, f3) = (named(&observed, "cx-f2b"), named(&observed, "cx-f3"));
    assert!(matches!(
        resolution(&observed, "anywhere"),
        Observation::AmbiguousOwned { .. }
    ));
    assert_eq!(
        resolution(&observed, "in-a"),
        &resolved_to("in-a", "cx-f2a")
    );
    assert_eq!(
        resolution(&observed, "in-b"),
        &resolved_to("in-b", "cx-f2b")
    );
    assert_eq!(
        loop_of(&world, "cell-loop-f2b", f2b),
        Some(the_loop(piece_loop))
    );

    // Merge the foreign piece with the third face keeping the least name:
    // when the piece's host goes, its one-face loop retires with it.
    let (away, kept) = if f2b < f3 {
        ("cx-f3", f2b)
    } else {
        ("cx-f2b", f3)
    };
    world.committed(
        "cell-merge",
        Plan {
            reads: keys(&["cx-f2b", "cx-f3"]),
            deletes: keys(&[away]),
            steps: vec![Step::Scoped(
                occurrence("cell-merge"),
                vec![ScopeStep::MergeLeast(
                    vec![Held::Name(f2b), Held::Name(f3)],
                    None,
                )],
            )],
            ..Plan::default()
        },
    );
    assert_eq!(
        world.names(
            "cell-kept",
            &[if away == "cx-f3" { "cx-f2b" } else { "cx-f3" }],
            2
        ),
        vec![kept]
    );
    let piece_now = read(
        &world,
        "cell-piece",
        vec![GroupRead::Loop(piece_loop, NameSelection::Exact)],
    );
    let expected = if away == "cx-f2b" {
        GroupAnswer::Retired
    } else {
        GroupAnswer::Exact(the_loop(piece_loop))
    };
    assert_eq!(piece_now, vec![expected]);
    let resolved = read(
        &world,
        "cell-resolve",
        vec![GroupRead::Loop(face_loop, NameSelection::Exact)],
    );
    assert_eq!(resolved, vec![GroupAnswer::Exact(the_loop(face_loop))]);
}
