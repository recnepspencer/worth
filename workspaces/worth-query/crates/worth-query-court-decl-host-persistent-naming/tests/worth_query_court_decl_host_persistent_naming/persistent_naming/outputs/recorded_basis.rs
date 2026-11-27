//! Phase 7B-2 `P1-A`: a discriminator references any name recorded on the
//! branch at the basis by its fixed 32-byte identity: a cutter the same
//! commit consumes, or an ancestor an earlier commit retired. The pieces'
//! recorded births decode those references back as members (and refuse as
//! another kind), the same before and after a Store reopen from the Journal
//! alone and from a checkpoint. A name this branch never recorded refuses.
use super::super::durable::{crash_and_compare, Holdings};
use super::super::world::home;
use super::*;
use worth_query_decl::facade::application_naming::{
    ApplicationNameDiscriminatorDecodeDenial, NameBirthDecodeDenial,
};
use worth_query_host::facade::application_naming::ExecutionNamingDenial;

/// The sided cut's wire form with its references typed as seams: decoding a
/// member reference through it refuses as a kind mismatch.
#[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.sided-cut.v1")]
enum SidedSeamCut {
    By {
        cutter: NameRef<Seam>,
        side: CutSide,
        along: Option<NameRef<Seam>>,
    },
}

fn piece(cutter: WorthQueryName<Member>, side: CutSide) -> WorthQueryOutputBirth {
    let cut = SidedCut::By {
        cutter: NameRef::Committed(cutter),
        side,
        along: None,
    };
    birth("recorded-basis-cut", "piece").with_discriminator(&cut)
}

fn authored(keys: &[&str]) -> Vec<Step> {
    let birth_of = |key: &&str| Step::Birth((*key).to_owned(), birth(key, "authored"));
    keys.iter().map(birth_of).collect()
}

#[test]
fn pieces_name_a_consumed_cutter_and_a_retired_ancestor_across_reopen() {
    let mut world = World::open_directory(home::location());
    let members = ["rb-tool", "rb-body", "rb-ancestor"];
    world.committed(
        "rb-create",
        Plan {
            creates: keys(&members),
            steps: authored(&members),
            ..Plan::default()
        },
    );
    let names = world.names("rb-names", &members, 0);
    let (tool, ancestor) = (names[0], names[2]);
    world.committed(
        "rb-retire",
        Plan {
            reads: keys(&["rb-ancestor"]),
            deletes: keys(&["rb-ancestor"]),
            steps: vec![Step::Retire(read("rb-ancestor"))],
            ..Plan::default()
        },
    );
    // The cut consumes its tool and names one piece after it, the other
    // after the ancestor retired by the commit before.
    world.committed(
        "rb-cut",
        Plan {
            reads: keys(&["rb-tool", "rb-body"]),
            deletes: keys(&["rb-tool", "rb-body"]),
            creates: keys(&["rb-p1", "rb-p2"]),
            steps: vec![
                Step::Retire(read("rb-tool")),
                Step::Split(
                    read("rb-body"),
                    vec![
                        ("rb-p1".into(), piece(tool, CutSide::Positive)),
                        ("rb-p2".into(), piece(ancestor, CutSide::Negative)),
                    ],
                ),
            ],
            ..Plan::default()
        },
    );
    let pieces = world.names("rb-pieces", &["rb-p1", "rb-p2"], 1);
    let token = "rb-births";
    probe::births::birth_of(token, "p1", pieces[0]);
    probe::births::birth_of(token, "p2", pieces[1]);
    let (observed, _) = world.probe_births(token, &["rb-p1", "rb-p2"], 0);
    for (label, cutter, side) in [
        ("p1", tool, CutSide::Positive),
        ("p2", ancestor, CutSide::Negative),
    ] {
        let recorded = births::born(&observed, label);
        let expected = SidedCut::By {
            cutter: NameRef::Committed(cutter),
            side,
            along: None,
        };
        assert_eq!(
            recorded.decode_discriminator::<SidedCut>(),
            Ok(Some(expected))
        );
        assert!(
            matches!(
                recorded.decode_discriminator::<SidedSeamCut>(),
                Err(NameBirthDecodeDenial::Discriminator(
                    ApplicationNameDiscriminatorDecodeDenial::NameKindMismatch { .. }
                ))
            ),
            "{label}: a member reference never decodes as a seam"
        );
    }
    let mut held = Holdings::default();
    held.keys = keys(&["rb-p1", "rb-p2"]);
    held.member("tool", tool);
    held.member("ancestor", ancestor);
    held.member("p1", pieces[0]);
    held.member("p2", pieces[1]);
    // The cut replays from the Journal alone, then reopens from a checkpoint.
    crash_and_compare(&mut world, &held, "recorded-basis-journal", false);
    crash_and_compare(&mut world, &held, "recorded-basis-checkpoint", true);
    // The reopened scheme still mints structurally: the piece's birth parts
    // find its name under any parent.
    let token = "rb-reopened-lookup";
    let birth = piece(tool, CutSide::Positive);
    probe::births::name_for_birth(token, "any-parent", birth, vec![ancestor]);
    let (observed, _) = world.probe_births(token, &["rb-p1"], 1);
    assert_eq!(births::minted(&observed, "any-parent"), Some(pieces[0]));
}

#[test]
fn a_name_this_branch_never_recorded_refuses() {
    let elsewhere = World::install();
    elsewhere.committed(
        "rb-foreign-create",
        Plan {
            creates: keys(&["rb-foreign"]),
            steps: authored(&["rb-foreign"]),
            ..Plan::default()
        },
    );
    let foreign = elsewhere.names("rb-foreign-name", &["rb-foreign"], 0)[0];
    let world = World::install();
    world.committed(
        "rb-unrecorded-create",
        Plan {
            creates: keys(&["rb-whole"]),
            steps: authored(&["rb-whole"]),
            ..Plan::default()
        },
    );
    let outcome = world.sculpt(
        "rb-unrecorded-cut",
        Plan {
            reads: keys(&["rb-whole"]),
            deletes: keys(&["rb-whole"]),
            creates: keys(&["rb-half"]),
            steps: vec![Step::Split(
                read("rb-whole"),
                vec![("rb-half".into(), piece(foreign, CutSide::Positive))],
            )],
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            output_refusal(&outcome),
            Some(WorthQueryOutputNameDenial::Assembly(
                ExecutionNamingDenial::UnresolvedReferencedName { digest, .. }
            )) if digest == foreign.as_bytes()
        ),
        "{outcome:?}"
    );
    assert!(world.live("rb-whole"), "the refused cut changed nothing");
}
