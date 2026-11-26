//! An installed invariant reads `name_of` and `birth_of` on committed
//! records and on records its own commit names: plain births, split pieces,
//! merge results kept in place or minted new, and a coupled producer's
//! outputs. Each answer is checked against an independent tracked reader
//! after the commit seals.
use super::*;
use crate::persistent_naming::outputs::sculpt::{Keeper, Step};
use crate::persistent_naming::outputs::{birth, keys, piece, read, SidedCut};
use worth_query_decl::facade::application_naming::NameRef;

/// The member the watch evaluated under `key`.
pub struct Evaluated {
    pub proposed: Result<WorthQueryName<Member>, String>,
    pub birth: Option<Result<RecordedNameBirth<Member>, String>>,
    pub committed: Option<Result<WorthQueryName<Member>, String>>,
    pub committed_birth: Option<Result<RecordedNameBirth<Member>, String>>,
}

pub fn evaluated(seen: &[Seen], key: &str) -> Evaluated {
    planned_too(seen, key);
    seen.iter()
        .find_map(|seen| match seen {
            Seen::Member {
                phase: Phase::Evaluated,
                key: k,
                proposed,
                birth,
                committed,
                committed_birth,
            } if k == key => Some(Evaluated {
                proposed: proposed.clone(),
                birth: birth.clone(),
                committed: committed.clone(),
                committed_birth: committed_birth.clone(),
            }),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the watch evaluates `{key}`: {seen:?}"))
}

/// Scope planning reads the same proposed name the evaluation reads.
fn planned_too(seen: &[Seen], key: &str) {
    let names = |wanted: Phase| {
        seen.iter().find_map(|seen| match seen {
            Seen::Member {
                phase,
                key: k,
                proposed,
                ..
            } if *phase == wanted && k == key => Some(proposed.clone()),
            _ => None,
        })
    };
    assert_eq!(names(Phase::Planned), names(Phase::Evaluated), "`{key}`");
}

pub fn gone(seen: &[Seen], key: &str) -> (Result<WorthQueryName<Member>, String>, bool, String) {
    seen.iter()
        .find_map(|seen| match seen {
            Seen::Gone {
                key: k,
                committed,
                reobserved,
                cross_view,
            } if k == key => Some((committed.clone(), *reobserved, cross_view.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the watch sees `{key}` gone: {seen:?}"))
}

fn birth_of(seen: &Evaluated) -> &RecordedNameBirth<Member> {
    match &seen.birth {
        Some(Ok(birth)) => birth,
        other => panic!("the proposed birth reads: {other:?}"),
    }
}

/// The committed view never sees a birth only the proposal records.
fn unseen_in_committed(seen: &Evaluated) {
    assert_eq!(seen.committed, None, "the record is new");
    match &seen.committed_birth {
        Some(Err(refusal)) => assert!(refusal.contains("UnknownName"), "{refusal}"),
        other => panic!("committed() must not see a pending birth: {other:?}"),
    }
}

#[test]
fn a_committed_members_name_and_birth_read_alike_in_both_views() {
    let world = WatchWorld::install();
    let [direct] = world.names("watch-committed", &["direct"], 0)[..] else {
        unreachable!()
    };
    world.take();
    world.edit_member("direct", 3, "watch-edit-direct");
    let seen = evaluated(&world.take(), "direct");
    assert_eq!(seen.proposed, Ok(direct));
    assert_eq!(seen.committed, Some(Ok(direct)));
    let birth = birth_of(&seen);
    assert_eq!(birth.name(), direct);
    assert_eq!(birth.decode_role::<MemberRole>(), Ok(MemberRole::Authored));
    assert_eq!(birth.decode_discriminator::<MemberCut>(), Ok(None));
    assert_eq!(seen.committed_birth.as_ref(), Some(&Ok(birth.clone())));
    let oracle = world.births("watch-committed-births", &[("direct", direct)], 1);
    assert_eq!(&oracle[0], birth, "the tracked reader reads the same birth");
}

#[test]
fn a_pending_birth_answers_with_the_name_and_birth_the_commit_records() {
    let world = WatchWorld::install();
    world.take();
    world.committed(
        "watch-plain",
        Plan {
            creates: keys(&["n1"]),
            steps: vec![Step::Birth("n1".into(), piece("n-1", 3, None))],
            ..Plan::default()
        },
    );
    let seen = evaluated(&world.take(), "n1");
    unseen_in_committed(&seen);
    let birth = birth_of(&seen);
    assert_eq!(
        birth.decode_discriminator::<MemberCut>(),
        Ok(Some(MemberCut::Side {
            side: 3,
            cutter: None
        }))
    );
    assert_eq!(
        (birth.occurrence_key(), birth.role_path()),
        ("n-1", "piece")
    );
    let [n1] = world.names("watch-plain-oracle", &["n1"], 0)[..] else {
        unreachable!()
    };
    assert_eq!(seen.proposed, Ok(n1), "the pending name is the minted name");
    let oracle = world.births("watch-plain-births", &[("n1", n1)], 1);
    assert_eq!(&oracle[0], birth, "the pending birth is the recorded birth");
}

#[test]
fn split_pieces_and_merge_results_read_their_pending_names_and_births() {
    let world = WatchWorld::install();
    world.committed("watch-seed", authored(&["s", "m1", "m2", "q1", "q2"]));
    let held = world.names("watch-seed", &["s", "m1", "m2", "q1", "q2"], 0);
    let [s, m1, m2, q1, q2] = held[..] else {
        unreachable!()
    };
    world.take();

    world.committed(
        "watch-split",
        Plan {
            reads: keys(&["s"]),
            deletes: keys(&["s"]),
            creates: keys(&["s1", "s2"]),
            steps: vec![Step::Split(
                read("s"),
                vec![
                    ("s1".into(), piece("cut-s", 1, None)),
                    ("s2".into(), piece("cut-s", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    let seen = world.take();
    let (before, reobserved, cross_view) = gone(&seen, "s");
    assert_eq!(before, Ok(s), "committed() still reads the split parent");
    assert!(
        !reobserved,
        "proposed() never answers the split parent as live"
    );
    assert!(cross_view.contains("ForeignView"), "{cross_view}");
    let pieces = world.names("watch-split-oracle", &["s1", "s2"], 1);
    for ((key, side), piece_name) in [("s1", 1), ("s2", 2)].into_iter().zip(pieces) {
        let watched = evaluated(&seen, key);
        unseen_in_committed(&watched);
        assert_eq!(watched.proposed, Ok(piece_name));
        let birth = birth_of(&watched);
        assert_eq!(birth.parents(), [s], "a piece's parent is the split name");
        assert_eq!(
            birth.decode_discriminator::<MemberCut>(),
            Ok(Some(MemberCut::Side { side, cutter: None }))
        );
    }

    world.committed(
        "watch-merge-keep",
        Plan {
            reads: keys(&["m1", "m2"]),
            deletes: keys(&["m2"]),
            moves: vec![("m1".into(), "a".into(), "b".into())],
            steps: vec![Step::Merge(
                vec![read("m1"), read("m2")],
                Keeper::InPlace(read("m1")),
            )],
            ..Plan::default()
        },
    );
    let seen = world.take();
    let keeper = evaluated(&seen, "m1");
    assert_eq!(keeper.proposed, Ok(m1), "KeepParentInPlace keeps the name");
    assert_eq!(keeper.committed, Some(Ok(m1)));
    assert_eq!(
        birth_of(&keeper).decode_role::<MemberRole>(),
        Ok(MemberRole::Authored)
    );
    let (before, reobserved, _) = gone(&seen, "m2");
    assert_eq!((before, reobserved), (Ok(m2), false));

    let minted_birth = birth("merge-q", "merged").with_discriminator(&MemberCut::Side {
        side: 0,
        cutter: None,
    });
    world.committed(
        "watch-merge-mint",
        Plan {
            reads: keys(&["q1", "q2"]),
            deletes: keys(&["q1", "q2"]),
            creates: keys(&["merged"]),
            steps: vec![Step::Merge(
                vec![read("q1"), read("q2")],
                Keeper::MintNew("merged".into(), minted_birth),
            )],
            ..Plan::default()
        },
    );
    let seen = world.take();
    let merged = evaluated(&seen, "merged");
    unseen_in_committed(&merged);
    let [minted] = world.names("watch-merge-oracle", &["merged"], 2)[..] else {
        unreachable!()
    };
    assert!(minted != q1 && minted != q2);
    assert_eq!(merged.proposed, Ok(minted));
    let birth = birth_of(&merged);
    let mut parents = vec![q1, q2];
    parents.sort();
    assert_eq!(birth.parents(), parents.as_slice());
    assert_eq!(birth.decode_role::<MemberRole>(), Ok(MemberRole::Merged));
    let oracle = world.births("watch-merge-births", &[("merged", minted)], 3);
    assert_eq!(&oracle[0], birth);
    for key in ["q1", "q2"] {
        assert!(!gone(&seen, key).1, "a merged-away parent is not live");
    }
}

#[test]
fn a_coupled_producers_births_are_read_in_the_composed_plan() {
    let world = WatchWorld::install();
    let outcome = world.shape("a", 5, "watch-shape");
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    let seen = world.take();
    let (primary, secondary) = (
        coupled::producer::primary_key("a"),
        coupled::producer::secondary_key("a"),
    );
    let names = world.names("watch-shape-oracle", &[&primary, &secondary], 0);
    let [first, second] = names[..] else {
        unreachable!()
    };
    let watched = evaluated(&seen, &primary);
    unseen_in_committed(&watched);
    assert_eq!(watched.proposed, Ok(first));
    let watched = evaluated(&seen, &secondary);
    unseen_in_committed(&watched);
    assert_eq!(watched.proposed, Ok(second));
    // The secondary's discriminator references the primary, born by the same
    // envelope: the proposed view admits that reference.
    assert_eq!(
        birth_of(&watched).decode_discriminator::<SidedCut>(),
        Ok(Some(SidedCut::By {
            cutter: NameRef::Committed(first),
            side: outputs::CutSide::Positive,
            along: None,
        }))
    );
}
