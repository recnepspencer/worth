//! Phase 7B-2 `P1-B`: commit-grouping identity determinism. Two sequential
//! splits landed in one commit or in two mint byte-identical names for every
//! surviving member under a structural-identity scheme, with the full split
//! lineage still resolving; a parent-inclusive scheme (seams) keeps telling
//! the two groupings apart by parent.
use super::seam_names::{link, members, read_name, seam, seam_read, MEMBERS};
use super::*;

const SURVIVORS: [&str; 3] = ["g-fb", "g-fa1", "g-fa2"];

fn cut(side: u64) -> WorthQueryOutputBirth {
    piece("grouping-cut", side, None)
}

fn split(parent: &str, children: &[(&str, u64)]) -> Plan {
    let created: Vec<&str> = children.iter().map(|(key, _)| *key).collect();
    Plan {
        reads: keys(&[parent]),
        deletes: keys(&[parent]),
        creates: keys(&created),
        steps: vec![Step::Split(
            read(parent),
            children
                .iter()
                .map(|(key, side)| ((*key).to_owned(), cut(*side)))
                .collect(),
        )],
        ..Plan::default()
    }
}

/// A world holding one authored member `g-f` and its name.
fn authored(prefix: &str) -> (World, WorthQueryName<Member>) {
    let world = World::install();
    world.committed(
        &format!("{prefix}-create"),
        Plan {
            creates: keys(&["g-f"]),
            steps: vec![Step::Birth("g-f".into(), birth("grouping-f", "authored"))],
            ..Plan::default()
        },
    );
    let whole = world.names(&format!("{prefix}-whole"), &["g-f"], 0)[0];
    (world, whole)
}

/// How `held` resolves under `selection`, its plural keys sorted.
fn resolved(world: &World, token: &str, held: WorthQueryName<Member>, all: bool) -> String {
    let selection = if all {
        NameSelection::AllDescendants
    } else {
        exact()
    };
    probe::hold_selecting(token, "held", held, selection);
    let observed = world.probe(token, &SURVIVORS, 9);
    match resolution(&observed, "held").clone() {
        probe::Observation::Plural { label, mut keys } => {
            keys.sort();
            format!("{:?}", probe::Observation::Plural { label, keys })
        }
        other => format!("{other:?}"),
    }
}

#[test]
fn one_commit_or_two_mint_identical_structural_names_with_full_lineage() {
    let (two, whole) = authored("g2");
    two.committed("g2-first", split("g-f", &[("g-fa", 1), ("g-fb", 2)]));
    let first_piece = two.names("g2-first-piece", &["g-fa"], 1)[0];
    two.committed("g2-second", split("g-fa", &[("g-fa1", 3), ("g-fa2", 4)]));

    let (one, one_whole) = authored("g1");
    assert_eq!(one_whole, whole, "the same authored birth");
    one.committed(
        "g1-both",
        split("g-f", &[("g-fb", 2), ("g-fa1", 3), ("g-fa2", 4)]),
    );

    let in_two = two.names("g2-survivors", &SURVIVORS, 2);
    let in_one = one.names("g1-survivors", &SURVIVORS, 2);
    assert_eq!(in_one, in_two, "byte-identical names for every survivor");

    // The lookup by birth parts agrees whichever parent each grouping lists.
    let token = "g-lookup";
    probe::births::name_for_birth(token, "via-first", cut(3), vec![first_piece]);
    let (two_lookup, _) = two.probe_births(token, &SURVIVORS, 3);
    let one_token = "g-lookup-one";
    probe::births::name_for_birth(one_token, "via-whole", cut(3), vec![whole]);
    let (one_lookup, _) = one.probe_births(one_token, &SURVIVORS, 3);
    assert_eq!(births::minted(&two_lookup, "via-first"), Some(in_two[1]));
    assert_eq!(births::minted(&one_lookup, "via-whole"), Some(in_one[1]));

    // Lineage resolves through both groupings to the same survivors.
    for all in [false, true] {
        assert_eq!(
            resolved(&one, &format!("g1-resolve-{all}"), whole, all),
            resolved(&two, &format!("g2-resolve-{all}"), whole, all),
        );
    }
    assert_eq!(
        resolved(&two, "g2-resolve-all", whole, true),
        format!(
            "{:?}",
            probe::Observation::Plural {
                label: "held".into(),
                keys: keys(&["g-fa1", "g-fa2", "g-fb"]),
            }
        )
    );
    assert_eq!(
        resolved(&two, "g2-resolve-first", first_piece, true),
        format!(
            "{:?}",
            probe::Observation::Plural {
                label: "held".into(),
                keys: keys(&["g-fa1", "g-fa2"]),
            }
        )
    );
}

/// `s` from `m1` to `m2`, then split once or twice; returns every live
/// seam's name, `[m1→m4, m2→m3, m2→m4]`.
fn seam_grouping(prefix: &str, two_commits: bool) -> Vec<WorthQueryName<Seam>> {
    let world = members();
    world.committed(
        &format!("{prefix}-link"),
        link("s", "m1", "m2", seam("grouping-weld", "seam", 0)),
    );
    let piece = |index| seam("grouping-weld-cut", "seam_piece", index);
    let split = |label: &str, held: (&str, &str), children: Vec<(&str, &str, &str, u64)>| Plan {
        reads: keys(&MEMBERS),
        seam_reads: vec![seam_read(label, held.0, held.1)],
        unlinks: vec![(held.0.into(), held.1.into())],
        seams: children
            .iter()
            .map(|(key, from, to, _)| ((*key).into(), (*from).into(), (*to).into()))
            .collect(),
        steps: vec![Step::SeamSplit(
            HeldSeam::Read(label.into()),
            children
                .iter()
                .map(|(key, _, _, index)| ((*key).into(), piece(*index)))
                .collect(),
        )],
        ..Plan::default()
    };
    let last = [
        ("sb", "m1", "m4", 2),
        ("sa1", "m2", "m3", 3),
        ("sa2", "m2", "m4", 4),
    ];
    if two_commits {
        let first = vec![("sa", "m1", "m3", 1), last[0]];
        world.committed(&format!("{prefix}-first"), split("s", ("m1", "m2"), first));
        let second = vec![last[1], last[2]];
        world.committed(
            &format!("{prefix}-second"),
            split("sa", ("m1", "m3"), second),
        );
    } else {
        world.committed(
            &format!("{prefix}-both"),
            split("s", ("m1", "m2"), last.to_vec()),
        );
    }
    [("m1", "m4"), ("m2", "m3"), ("m2", "m4")]
        .iter()
        .map(|(from, to)| read_name(&world, &format!("{prefix}-{from}-{to}"), from, to))
        .collect()
}

#[test]
fn a_parent_inclusive_scheme_keeps_telling_the_groupings_apart() {
    let two = seam_grouping("gs2", true);
    let one = seam_grouping("gs1", false);
    // The seam split once in both groupings has the same parent and name;
    // the seams split from different parents keep different names.
    assert_eq!(one[0], two[0]);
    assert_ne!(one[1], two[1]);
    assert_ne!(one[2], two[2]);
}
