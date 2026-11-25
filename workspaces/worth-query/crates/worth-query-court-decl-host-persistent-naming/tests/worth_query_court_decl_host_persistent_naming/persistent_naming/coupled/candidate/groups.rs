//! #361: a later edit-envelope stage reads persistent groups on the
//! envelope's unpublished candidate. `group_name_of` and `resolve_group`
//! answer from the commit basis while the candidate claims no partition of
//! the kind's scheme and leaves every member the answer rests on live. An
//! earlier stage's partition or dissolve, a deleted group member, a member
//! name the candidate ended, and a held group that is no longer itself live
//! each refuse typed; the stage cannot seal and nothing commits. A later
//! stage that claims a scheme an earlier stage read on the candidate refuses
//! the whole envelope.
use super::super::super::groups::{declare, group, loop_of, loops, resolve_loop, seeded, the_loop};
use super::super::super::outputs::sculpt::{GroupAnswer, GroupRead, Held};
use super::*;
use worth_query_host::facade::application_naming::NameSelection;
use GroupRole::{Host, Member as Plain};

/// Members `h`, `x`, `y`, `z`; one loop hosted by `h` holding `x`.
fn looped(token: &str) -> (World, Vec<WorthQueryName<Member>>, WorthQueryName<Loop>) {
    let (world, n) = seeded(token, &["h", "x", "y", "z"]);
    declare(
        &world,
        &format!("{token}-loop"),
        loops(vec![group(&[(Host, n[0]), (Plain, n[1])])], &[]),
    );
    let whole = loop_of(&world, &format!("{token}-whole"), n[0]).expect("h hosts a loop");
    (world, n, whole)
}

/// A stage whose decision makes `reads` and writes nothing.
fn reading(token: &str, reads: Vec<GroupRead>) -> Part {
    sculpted(
        token,
        Plan {
            group_reads: reads,
            ..Plan::default()
        },
    )
}

fn of(name: WorthQueryName<Member>) -> GroupRead {
    GroupRead::LoopOf(Held::Name(name))
}

fn refused_with(answer: &GroupAnswer, variant: &str) -> bool {
    matches!(answer, GroupAnswer::Refused(refusal) if refusal.starts_with(variant))
}

fn uncommitted(outcome: &Envelope) {
    assert!(
        !matches!(outcome, Envelope::Committed(_)),
        "a refused candidate group read never seals: {outcome:?}"
    );
}

#[test]
fn a_later_stage_reads_groups_its_earlier_stages_left_unchanged() {
    let token = "cand-groups";
    let (world, n, whole) = looped(token);
    let answers = format!("{token}-read");
    let outcome = world.parts(
        token,
        vec![
            // A field edit of the host, then an unrelated named creation:
            // neither claims the scheme nor ends a member.
            edited("h", 5),
            sculpted("cand-groups-1", created("fresh", "cand-groups-fresh")),
            reading(
                &answers,
                vec![
                    of(n[0]),
                    of(n[1]),
                    of(n[2]),
                    GroupRead::Loop(whole, NameSelection::Exact),
                    GroupRead::Loop(whole, NameSelection::AllDescendants),
                ],
            ),
        ],
    );
    assert!(matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
    let the = Some(the_loop(whole));
    assert_eq!(
        sculpt::group_answers(&answers),
        vec![
            GroupAnswer::Of(the),
            GroupAnswer::Of(the),
            GroupAnswer::Of(None),
            GroupAnswer::Exact(the_loop(whole)),
            GroupAnswer::Exact(the_loop(whole)),
        ]
    );
    // Every stage committed, and the committed groups agree with the read.
    assert_eq!(world.member_value("h"), 5);
    assert!(world.live("fresh"));
    assert_eq!(
        loop_of(&world, &format!("{token}-after"), n[1]),
        Some(whole)
    );
}

#[test]
fn an_earlier_partition_or_dissolve_of_the_scheme_refuses_later_group_reads() {
    let token = "cand-groups-claim";
    let (world, n, whole) = looped(token);
    let claims = [
        ("new", loops(vec![group(&[(Host, n[2])])], &[])),
        ("dissolve", loops(Vec::new(), &[whole])),
    ];
    for (label, claim) in claims {
        let answers = format!("{token}-{label}-read");
        let outcome = world.parts(
            &format!("{token}-{label}"),
            vec![
                sculpted(
                    &format!("{token}-{label}-1"),
                    Plan {
                        steps: vec![Step::Groups(claim)],
                        ..Plan::default()
                    },
                ),
                reading(
                    &answers,
                    vec![
                        of(n[0]),
                        of(n[3]),
                        GroupRead::Loop(whole, NameSelection::Exact),
                    ],
                ),
            ],
        );
        let read = sculpt::group_answers(&answers);
        assert_eq!(read.len(), 3, "{label}: {read:?}");
        assert!(
            read.iter().all(|answer| refused_with(answer, "CandidateGroupState")),
            "{label}: every read of the claimed scheme refuses, even of an ungrouped member: {read:?}"
        );
        uncommitted(&outcome);
        // Zero effects: the claim never committed.
        let check = format!("{token}-{label}-after");
        assert_eq!(loop_of(&world, &check, n[0]), Some(whole), "{label}");
        assert_eq!(
            loop_of(&world, &format!("{check}-y"), n[2]),
            None,
            "{label}"
        );
    }
}

#[test]
fn an_earlier_member_deletion_refuses_every_read_of_its_group() {
    let token = "cand-groups-deleted";
    let (world, n, whole) = looped(token);
    let answers = format!("{token}-read");
    let outcome = world.parts(
        token,
        vec![
            // Deleting `x` shrinks the loop at commit without any claim.
            sculpted(&format!("{token}-1"), retired("x", n[1])),
            reading(
                &answers,
                vec![
                    of(n[0]),
                    GroupRead::Loop(whole, NameSelection::Exact),
                    of(n[1]),
                    of(n[2]),
                ],
            ),
        ],
    );
    let read = sculpt::group_answers(&answers);
    assert_eq!(read.len(), 4, "{read:?}");
    for answer in &read[..3] {
        assert!(
            refused_with(answer, "EndedInCandidate(Entity("),
            "a read resting on the deleted member never answers stale membership: {read:?}"
        );
    }
    // A member of no group whose record the candidate left live answers.
    assert_eq!(read[3], GroupAnswer::Of(None));
    uncommitted(&outcome);
    assert!(world.live("x"), "the deletion never committed");
    assert_eq!(
        loop_of(&world, &format!("{token}-after"), n[0]),
        Some(whole)
    );
}

#[test]
fn a_member_an_earlier_stage_created_or_a_committed_ended_member_name_refuses() {
    let token = "cand-groups-unborn";
    let (world, n, _) = looped(token);
    // A member an earlier stage created has no published name, so no group
    // read can be keyed on it: the stage's name read refuses first.
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                &format!("{token}-1"),
                created("fresh", "cand-groups-unborn"),
            ),
            sculpted(
                &format!("{token}-read"),
                Plan {
                    reads: keys(&["fresh"]),
                    group_reads: vec![GroupRead::LoopOf(Held::Read("fresh".into()))],
                    ..Plan::default()
                },
            ),
        ],
    );
    uncommitted(&outcome);
    assert!(!world.live("fresh"), "zero effects");
    // A held member name whose record a commit ended rests on name history
    // the candidate may restore: committed it answers no group, on the
    // candidate it refuses.
    world.committed(&format!("{token}-retire"), retired("z", n[3]));
    assert_eq!(loop_of(&world, &format!("{token}-z"), n[3]), None);
    let answers = format!("{token}-history-read");
    let outcome = world.parts(
        &format!("{token}-history"),
        vec![edited("h", 6), reading(&answers, vec![of(n[3])])],
    );
    assert!(
        refused_with(&sculpt::group_answers(&answers)[0], "CandidateNameHistory"),
        "{:?}",
        sculpt::group_answers(&answers)
    );
    uncommitted(&outcome);
    assert_ne!(world.member_value("h"), 6, "zero effects");
}

/// The loop over `h`, `x` and `y` loses its host and splits into one loop
/// per remaining member. Committed, `Exact` on the ended loop is
/// `Ambiguous`; on the candidate it rests on group history and refuses, even
/// after an earlier stage deleted one sibling's member: the ambiguity never
/// collapses to the survivor. A dissolved loop refuses the same way.
#[test]
fn held_group_history_refuses_on_the_candidate_and_never_collapses() {
    let token = "cand-groups-split";
    let (world, n) = seeded(token, &["h", "x", "y"]);
    declare(
        &world,
        &format!("{token}-0"),
        loops(
            vec![group(&[(Host, n[0]), (Plain, n[1]), (Plain, n[2])])],
            &[],
        ),
    );
    let whole = loop_of(&world, &format!("{token}-whole"), n[0]).unwrap();
    world.committed(
        &format!("{token}-1"),
        Plan {
            reads: keys(&["h"]),
            deletes: keys(&["h"]),
            steps: vec![Step::Groups(loops(
                vec![group(&[(Host, n[1])]), group(&[(Host, n[2])])],
                &[],
            ))],
            ..Plan::default()
        },
    );
    let committed = resolve_loop(
        &world,
        &format!("{token}-exact"),
        whole,
        NameSelection::Exact,
    );
    assert!(
        matches!(committed, GroupAnswer::Ambiguous(_)),
        "{committed:?}"
    );
    let left = loop_of(&world, &format!("{token}-x"), n[1]).unwrap();
    let answers = format!("{token}-read");
    let outcome = world.parts(
        &format!("{token}-trap"),
        vec![
            sculpted(&format!("{token}-trap-1"), retired("y", n[2])),
            reading(
                &answers,
                vec![
                    GroupRead::Loop(whole, NameSelection::Exact),
                    GroupRead::Loop(whole, NameSelection::AllDescendants),
                ],
            ),
        ],
    );
    let read = sculpt::group_answers(&answers);
    assert!(
        read.iter()
            .all(|answer| refused_with(answer, "CandidateNameHistory")),
        "never `Exact` the survivor: {read:?}"
    );
    uncommitted(&outcome);
    assert!(world.live("y"), "zero effects");
    // A retired group refuses on the candidate too.
    declare(
        &world,
        &format!("{token}-dissolve"),
        loops(Vec::new(), &[left]),
    );
    let answers = format!("{token}-retired-read");
    let outcome = world.parts(
        &format!("{token}-retired"),
        vec![
            edited("x", 7),
            reading(&answers, vec![GroupRead::Loop(left, NameSelection::Exact)]),
        ],
    );
    assert!(
        refused_with(&sculpt::group_answers(&answers)[0], "CandidateNameHistory"),
        "{:?}",
        sculpt::group_answers(&answers)
    );
    uncommitted(&outcome);
}

/// A later stage's partition of a scheme an earlier stage read on the
/// candidate would regroup what that read answered: the envelope refuses,
/// and nothing commits.
#[test]
fn a_later_claim_of_a_scheme_read_on_the_candidate_refuses_the_envelope() {
    let token = "cand-groups-later";
    let (world, n, whole) = looped(token);
    let answers = format!("{token}-read");
    let outcome = world.parts(
        token,
        vec![
            edited("h", 8),
            reading(&answers, vec![of(n[0])]),
            sculpted(
                &format!("{token}-claim"),
                Plan {
                    steps: vec![Step::Groups(loops(vec![group(&[(Host, n[2])])], &[]))],
                    ..Plan::default()
                },
            ),
        ],
    );
    assert_eq!(
        sculpt::group_answers(&answers),
        vec![GroupAnswer::Of(Some(the_loop(whole)))]
    );
    uncommitted(&outcome);
    assert!(
        format!("{outcome:?}").contains("claimed group scheme"),
        "{outcome:?}"
    );
    assert_ne!(world.member_value("h"), 8, "zero effects");
    assert_eq!(loop_of(&world, &format!("{token}-y"), n[2]), None);
}
