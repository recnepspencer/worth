//! Slice 5D: an installed operation reads a live seam's name with
//! `relation_name_of`, resolves held seam names with `resolve_relation`, and
//! carries one seam through `Create`, `Preserve`, `Split`, every `Merge`
//! keeper and `Retire` by the names its own decision read.
use super::*;

pub const MEMBERS: [&str; 4] = ["m1", "m2", "m3", "m4"];

pub fn seam(occurrence: &str, role: &str, index: u64) -> WorthQueryOutputBirth {
    birth(occurrence, role).with_discriminator(&SeamSide { index })
}

/// A world of four named members with no seam yet.
pub fn members() -> World {
    let world = World::install();
    // One plan under one token for every court: a concurrent court
    // re-authoring it changes nothing.
    world.committed(
        "seam-members",
        Plan {
            creates: keys(&MEMBERS),
            steps: MEMBERS
                .iter()
                .map(|key| Step::Birth((*key).to_owned(), birth(&format!("occ-{key}"), "authored")))
                .collect(),
            ..Plan::default()
        },
    );
    world
}

/// Links the seam `key` from `from` to `to`, born as `named`.
pub fn link(key: &str, from: &str, to: &str, named: WorthQueryOutputBirth) -> Plan {
    Plan {
        reads: keys(&[from, to]),
        seams: vec![(key.into(), from.into(), to.into())],
        steps: vec![Step::SeamBirth(key.into(), named)],
        ..Plan::default()
    }
}

pub fn seam_read(label: &str, from: &str, to: &str) -> (String, String, String) {
    (label.into(), from.into(), to.into())
}

pub fn ends(from: &str, to: &str) -> (String, String) {
    (from.into(), to.into())
}

/// Reads the name of the live seam from `from` to `to` in a sealing probe.
pub fn read_name(world: &World, token: &str, from: &str, to: &str) -> WorthQueryName<Seam> {
    probe::seams::name(token, "read", from, to);
    let (observed, ended) = world.probe_seams(token, &MEMBERS, 0);
    assert!(
        ended.contains("Committed"),
        "a live seam's name seals: {ended}"
    );
    seam_named(&observed, "read")
}

/// Resolves `held` under `selection` and returns the observation and how
/// the probe run ended.
pub fn resolve(
    world: &World,
    token: &str,
    held: WorthQueryName<Seam>,
    selection: NameSelection,
) -> (SeamObservation, String) {
    probe::seams::hold(token, "held", held, selection);
    let (observed, ended) = world.probe_seams(token, &MEMBERS, 0);
    (seam_resolution(&observed, "held").clone(), ended)
}

#[test]
fn a_decision_reads_a_live_seams_name_and_resolves_it_exactly() {
    let world = members();
    world.committed("sr-link", link("s", "m1", "m2", seam("weld", "seam", 1)));
    let name = read_name(&world, "sr-read", "m1", "m2");
    let (resolved, ended) = resolve(&world, "sr-resolve", name, exact());
    assert_eq!(resolved, seam_between("held", "m1", "m2"));
    assert!(
        ended.contains("Committed"),
        "a resolution resting on one live seam seals: {ended}"
    );
    // A second seam of the same occurrence and role, told apart by its
    // discriminator alone, carries a different name.
    world.committed("sr-link-2", link("t", "m3", "m4", seam("weld", "seam", 2)));
    assert_ne!(read_name(&world, "sr-read-2", "m3", "m4"), name);
}

#[test]
fn a_held_seam_follows_create_preserve_split_every_merge_keeper_and_retire() {
    let world = members();
    world.committed("sl-create", link("s", "m1", "m2", seam("weld", "seam", 1)));
    let s = read_name(&world, "sl-created", "m1", "m2");

    // `Preserve`: the run retires the name it read with the seam it unlinks
    // and re-mints the same occurrence and role on a seam it links.
    world.committed(
        "sl-preserve",
        Plan {
            reads: keys(&["m1", "m2", "m3"]),
            seam_reads: vec![seam_read("s", "m1", "m2")],
            unlinks: vec![("m1".into(), "m2".into())],
            seams: vec![("s2".into(), "m1".into(), "m3".into())],
            steps: vec![
                Step::SeamRetire(HeldSeam::Read("s".into())),
                Step::SeamBirth("s2".into(), seam("weld", "seam", 1)),
            ],
            ..Plan::default()
        },
    );
    assert_eq!(read_name(&world, "sl-preserved", "m1", "m3"), s);
    assert_eq!(
        resolve(&world, "sl-preserved-held", s, exact()).0,
        seam_between("held", "m1", "m3")
    );

    // `Split` into two seams told apart by discriminator alone.
    world.committed(
        "sl-split",
        Plan {
            reads: keys(&MEMBERS),
            seam_reads: vec![seam_read("s", "m1", "m3")],
            unlinks: vec![("m1".into(), "m3".into())],
            seams: vec![
                ("p1".into(), "m1".into(), "m2".into()),
                ("p2".into(), "m1".into(), "m4".into()),
            ],
            steps: vec![Step::SeamSplit(
                HeldSeam::Read("s".into()),
                vec![
                    ("p1".into(), seam("cut", "seam_piece", 1)),
                    ("p2".into(), seam("cut", "seam_piece", 2)),
                ],
            )],
            ..Plan::default()
        },
    );
    let (p1, p2) = (
        read_name(&world, "sl-piece-1", "m1", "m2"),
        read_name(&world, "sl-piece-2", "m1", "m4"),
    );
    let mut pieces = vec![p1, p2];
    pieces.sort();
    assert_eq!(
        resolve(&world, "sl-split-exact", s, exact()).0,
        SeamObservation::Ambiguous {
            label: "held".into(),
            candidates: pieces,
        },
        "Exact never picks a least member"
    );
    let SeamObservation::Plural {
        ends: mut found, ..
    } = resolve(&world, "sl-split-all", s, NameSelection::AllDescendants).0
    else {
        panic!("AllDescendants returns both pieces")
    };
    found.sort();
    assert_eq!(found, vec![ends("m1", "m2"), ends("m1", "m4")]);

    // `Merge` keeping one parent in place: the other's name follows it.
    world.committed(
        "sl-merge-in-place",
        Plan {
            reads: keys(&["m1", "m2", "m4"]),
            seam_reads: vec![seam_read("p1", "m1", "m2"), seam_read("p2", "m1", "m4")],
            unlinks: vec![("m1".into(), "m4".into())],
            steps: vec![Step::SeamMerge(
                vec![HeldSeam::Read("p1".into()), HeldSeam::Read("p2".into())],
                SeamKeeper::InPlace(HeldSeam::Read("p1".into())),
            )],
            ..Plan::default()
        },
    );
    for (token, held) in [("sl-in-place-p2", p2), ("sl-in-place-s", s)] {
        assert_eq!(
            resolve(&world, token, held, exact()).0,
            seam_between("held", "m1", "m2")
        );
    }

    // `Merge` carrying the keeper's name onto a seam the run links.
    world.committed(
        "sl-link-x",
        link("x", "m3", "m4", seam("weld-x", "seam", 1)),
    );
    let x = read_name(&world, "sl-x", "m3", "m4");
    world.committed(
        "sl-merge-onto",
        Plan {
            reads: keys(&MEMBERS),
            seam_reads: vec![seam_read("p1", "m1", "m2"), seam_read("x", "m3", "m4")],
            unlinks: vec![("m1".into(), "m2".into()), ("m3".into(), "m4".into())],
            seams: vec![("k".into(), "m2".into(), "m3".into())],
            steps: vec![Step::SeamMerge(
                vec![HeldSeam::Read("p1".into()), HeldSeam::Read("x".into())],
                SeamKeeper::Onto(HeldSeam::Read("p1".into()), "k".into()),
            )],
            ..Plan::default()
        },
    );
    assert_eq!(read_name(&world, "sl-onto", "m2", "m3"), p1);
    assert_eq!(
        resolve(&world, "sl-onto-x", x, exact()).0,
        seam_between("held", "m2", "m3")
    );

    // `Merge` minting a new name that carries both parents.
    world.committed(
        "sl-link-y",
        link("y", "m3", "m1", seam("weld-y", "seam", 1)),
    );
    let y = read_name(&world, "sl-y", "m3", "m1");
    world.committed(
        "sl-merge-mint",
        Plan {
            reads: keys(&MEMBERS),
            seam_reads: vec![seam_read("p1", "m2", "m3"), seam_read("y", "m3", "m1")],
            unlinks: vec![("m2".into(), "m3".into()), ("m3".into(), "m1".into())],
            seams: vec![("mn".into(), "m4".into(), "m1".into())],
            steps: vec![Step::SeamMerge(
                vec![HeldSeam::Read("p1".into()), HeldSeam::Read("y".into())],
                SeamKeeper::MintNew("mn".into(), seam("fuse", "seam_piece", 9)),
            )],
            ..Plan::default()
        },
    );
    let minted = read_name(&world, "sl-minted", "m4", "m1");
    assert!(minted != p1 && minted != y, "the survivor's name is new");
    for (token, held) in [("sl-mint-p1", p1), ("sl-mint-y", y), ("sl-mint-s", s)] {
        assert_eq!(
            resolve(&world, token, held, exact()).0,
            seam_between("held", "m4", "m1")
        );
    }

    // `Retire` ends the survivor and every name that resolved to it.
    world.committed(
        "sl-retire",
        Plan {
            reads: keys(&["m4", "m1"]),
            seam_reads: vec![seam_read("mn", "m4", "m1")],
            unlinks: vec![("m4".into(), "m1".into())],
            steps: vec![Step::SeamRetire(HeldSeam::Read("mn".into()))],
            ..Plan::default()
        },
    );
    let retired = SeamObservation::Retired {
        label: "held".into(),
    };
    for (token, held) in [
        ("sl-retired-mn", minted),
        ("sl-retired-s", s),
        ("sl-retired-x", x),
    ] {
        assert_eq!(resolve(&world, token, held, exact()).0, retired);
    }
}
