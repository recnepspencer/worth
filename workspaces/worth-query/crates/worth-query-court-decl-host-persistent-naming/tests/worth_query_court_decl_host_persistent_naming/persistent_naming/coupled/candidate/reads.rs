//! Candidate-basis reads that hold: a later stage reads the names of records
//! an earlier stage left live at the same identity (unchanged, field-edited
//! or re-owned), seals, and the envelope commits.
use super::*;
use probe::Observation;

/// Seeded `direct` and `foreign`, and seam `s1` from `direct` to `nested`:
/// their names as a committed probe reads them.
fn basis(
    world: &World,
    token: &str,
) -> (
    WorthQueryName<Member>,
    WorthQueryName<Member>,
    WorthQueryName<Seam>,
) {
    let names = world.names(&format!("{token}-basis"), &["direct", "foreign"], 0);
    let seam = world.seam_name(&format!("{token}-basis-seam"), "direct", "nested");
    (names[0], names[1], seam)
}

#[test]
fn a_later_stage_reads_unchanged_and_field_edited_names_at_the_commit_basis() {
    let token = "cand-reads";
    let world = seamed(token);
    let (direct, foreign, s1) = basis(&world, token);
    probe::hold(token, "direct", direct);
    probe::hold_in(token, "direct-in-a", direct, outputs::exact(), "a");
    probe::hold_in(token, "foreign-in-b", foreign, outputs::exact(), "b");
    probe::hold_in(token, "direct-in-b", direct, outputs::exact(), "b");
    probe::seams::name(token, "s1", "direct", "nested");
    probe::seams::hold(token, "s1-held", s1, outputs::exact());
    let outcome = world.parts(
        token,
        vec![
            // A field edit of a record the probe reads, then an unrelated
            // named creation: neither changes any name the probe reads.
            edited("direct", 5),
            sculpted("cand-reads-1", created("fresh", "cand-reads-fresh")),
            probed(token, &["direct", "nested", "foreign"]),
        ],
    );
    assert!(matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
    let observed = probe::take(token);
    assert_eq!(outputs::named(&observed, "direct"), direct);
    assert_eq!(outputs::named(&observed, "foreign"), foreign);
    assert_eq!(
        outputs::resolution(&observed, "direct"),
        &outputs::resolved_to("direct", "direct")
    );
    assert_eq!(
        outputs::resolution(&observed, "direct-in-a"),
        &outputs::resolved_to("direct-in-a", "direct")
    );
    assert_eq!(
        outputs::resolution(&observed, "foreign-in-b"),
        &outputs::resolved_to("foreign-in-b", "foreign")
    );
    assert_eq!(
        outputs::resolution(&observed, "direct-in-b"),
        &Observation::NotInOwner {
            label: "direct-in-b".into(),
            candidates: vec![(direct, vec!["a".into()])],
        }
    );
    let seams = probe::seams::take(token);
    assert_eq!(outputs::seam_named(&seams, "s1"), s1);
    assert_eq!(
        outputs::seam_resolution(&seams, "s1-held"),
        &outputs::seam_between("s1-held", "direct", "nested")
    );
    // Every stage's effect committed in the one envelope.
    assert_eq!(world.member_value("direct"), 5);
    assert_eq!(world.model_value("a"), 700);
    assert!(world.live("fresh"));
}

/// Stage 1 moves `direct` from model `a` to model `b` by re-linking its
/// owner relation; the member itself is untouched. Stage 2's owner-scoped
/// resolution reads the owner chain on the candidate: `b` holds it, `a`
/// no longer does.
#[test]
fn a_same_candidate_re_owning_is_what_resolve_in_sees() {
    let token = "cand-relink";
    let world = World::install();
    let direct = world.names(&format!("{token}-basis"), &["direct"], 0)[0];
    probe::hold_in(token, "in-b", direct, outputs::exact(), "b");
    probe::hold_in(token, "in-a", direct, outputs::exact(), "a");
    probe::hold(token, "anywhere", direct);
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "cand-relink-1",
                Plan {
                    plain_reads: keys(&["direct"]),
                    moves: vec![("direct".into(), "a".into(), "b".into())],
                    ..Plan::default()
                },
            ),
            probed(token, &["direct"]),
        ],
    );
    assert!(matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
    let observed = probe::take(token);
    assert_eq!(outputs::named(&observed, "direct"), direct);
    assert_eq!(
        outputs::resolution(&observed, "in-b"),
        &outputs::resolved_to("in-b", "direct")
    );
    assert_eq!(
        outputs::resolution(&observed, "in-a"),
        &Observation::NotInOwner {
            label: "in-a".into(),
            candidates: vec![(direct, vec!["b".into()])],
        }
    );
    assert_eq!(
        outputs::resolution(&observed, "anywhere"),
        &outputs::resolved_to("anywhere", "direct")
    );
    // The committed world agrees with what the candidate read showed.
    let after = format!("{token}-after");
    probe::hold_in(&after, "in-b", direct, outputs::exact(), "b");
    let observed = world.probe(&after, &["direct"], 1);
    assert_eq!(
        outputs::resolution(&observed, "in-b"),
        &outputs::resolved_to("in-b", "direct")
    );
}

/// An in-place merge keeper keeps its own lineage: a later stage still
/// reads its name and resolves it, while the merged-away parent's name
/// refuses typed (its record ended in the candidate).
#[test]
fn an_in_place_merge_keeper_keeps_its_name_on_the_candidate() {
    let token = "cand-keeper";
    let world = World::install();
    let names = world.names(&format!("{token}-basis"), &["direct", "loose"], 0);
    let (direct, loose) = (names[0], names[1]);
    probe::hold(token, "direct", direct);
    probe::hold(token, "loose", loose);
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "cand-keeper-1",
                Plan {
                    plain_reads: keys(&["direct", "loose"]),
                    deletes: keys(&["loose"]),
                    steps: vec![Step::Merge(
                        vec![sculpt::Held::Name(direct), sculpt::Held::Name(loose)],
                        sculpt::Keeper::InPlace(sculpt::Held::Name(direct)),
                    )],
                    ..Plan::default()
                },
            ),
            probed(token, &["direct"]),
        ],
    );
    let observed = probe::take(token);
    assert_eq!(outputs::named(&observed, "direct"), direct);
    assert_eq!(
        outputs::resolution(&observed, "direct"),
        &outputs::resolved_to("direct", "direct")
    );
    assert!(
        entity_denial(
            "EndedInCandidate",
            refused(outputs::resolution(&observed, "loose"))
        ),
        "{observed:?}"
    );
    nothing_committed(&world, &outcome, &[]);
    assert!(world.live("loose"), "the merge never committed");
}
