//! Fresh-installation admission and bare-identity refusal (Arch 10, 21).
//!
//! A reopened Directory Home is a fresh installation of the same schema
//! lineage. Retained names it recorded readmit only through their persisted
//! birth events: they resolve to their exact records and decode their typed
//! birth. The typed API has no raw-bytes constructor, so a well-formed
//! 32-byte identity with no birth here reaches this store only as a held
//! reference: names genuinely minted by an unrelated installation are that
//! bare identity, and every read of them refuses typed `UnknownName`.
use super::super::groups::loop_of;
use super::super::outputs::sculpt::{GroupDecl, GroupMember, GroupRead, Partition, Plan, Step};
use super::super::outputs::{birth, keys, piece, World};
use super::super::probe::{births::BirthObservation, seams::SeamObservation, Observation};
use super::super::world::home;
use super::*;

/// Commits a named member, a cut piece of it, a named seam and a loop, and
/// returns what an application keeps: their names and probed keys.
pub(super) fn named_world(world: &World, tag: &str) -> Holdings {
    let (member, cut) = (format!("{tag}-m"), format!("{tag}-p"));
    let seam = format!("{tag}-s");
    world.committed(
        &format!("{tag}-create"),
        Plan {
            creates: vec![member.clone()],
            steps: vec![Step::Birth(member.clone(), birth(&member, "authored"))],
            ..Plan::default()
        },
    );
    let parent = world.names(&format!("{tag}-parent"), &[&member], 0)[0];
    world.committed(
        &format!("{tag}-piece"),
        Plan {
            reads: vec![member.clone()],
            creates: vec![cut.clone()],
            seams: vec![(seam.clone(), member.clone(), cut.clone())],
            steps: vec![
                Step::Birth(cut.clone(), piece(&cut, 1, Some(parent))),
                Step::SeamBirth(seam.clone(), birth(&seam, "authored")),
                Step::Groups(GroupDecl::Loop(Partition {
                    groups: vec![vec![
                        (GroupRole::Host, GroupMember::Created(cut.clone())),
                        (GroupRole::Member, GroupMember::Held(read_name(parent))),
                    ]],
                    dissolve: Vec::new(),
                })),
            ],
            ..Plan::default()
        },
    );
    let piece_name = world.names(&format!("{tag}-pieces"), &[&cut], 1)[0];
    let pairs = vec![("seam".to_owned(), member.clone(), cut.clone())];
    let _ = world.try_sculpt(
        &format!("{tag}-seam-name"),
        Plan {
            reads: vec![member.clone(), cut.clone()],
            seam_reads: pairs.clone(),
            ..Plan::default()
        },
    );
    let seam_name = super::super::outputs::sculpt::seam_names(&format!("{tag}-seam-name"))["seam"];
    let the_loop = loop_of(world, &format!("{tag}-loop"), piece_name).expect("a loop");
    Holdings {
        keys: keys(&[member.as_str(), cut.as_str()]),
        members: vec![("member".into(), parent), ("piece".into(), piece_name)],
        seams: vec![("seam".into(), seam_name)],
        seam_pairs: pairs,
        loops: vec![("loop".into(), the_loop)],
    }
}

fn read_name(name: WorthQueryName<Member>) -> super::super::outputs::sculpt::Held {
    super::super::outputs::sculpt::Held::Name(name)
}

#[test]
fn a_fresh_installation_readmits_recorded_names_and_refuses_bare_identities() {
    let mut world = World::open_directory(home::location());
    let mut held = named_world(&world, "fa");
    // Group names across a checkpoint reopen are `groups_reopen`'s court.
    held.loops.clear();
    let before = served(&world, &held, "fa-before");
    world.checkpoint();
    world.crash_and_reopen();
    assert_eq!(
        served(&world, &held, "fa-after"),
        before,
        "recorded names readmit"
    );
    assert!(
        before.iter().any(|line| line.starts_with("Ok(Some(Side {")),
        "the piece's persisted birth decodes its typed discriminator: {before:#?}"
    );

    // Identities minted by an unrelated installation: no birth event here.
    let elsewhere = World::install();
    let bare = named_world(&elsewhere, "fb");
    let token = "fa-bare";
    for (label, name) in &bare.members {
        probe::hold(token, label, *name);
        probe::hold_in(
            token,
            &format!("{label}-in-a"),
            *name,
            NameSelection::Exact,
            "a",
        );
        probe::births::birth_of(token, label, *name);
    }
    for (label, name) in &bare.seams {
        probe::seams::hold(token, label, *name, NameSelection::Exact);
        probe::births::seam_birth_of(token, label, *name);
    }
    let observed = world.probe(token, &[], 0);
    assert_eq!(observed.len(), 4, "{observed:?}");
    for observation in &observed {
        let Observation::ResolveRefused { refusal, .. } = observation else {
            panic!("a bare member identity never resolves: {observation:?}");
        };
        assert!(refusal.contains("UnknownName"), "{refusal}");
    }
    let seams = probe::seams::take(token);
    assert!(
        matches!(seams.as_slice(), [SeamObservation::ResolveRefused { refusal, .. }] if refusal.contains("UnknownName")),
        "{seams:?}"
    );
    let births = probe::births::take(token);
    assert_eq!(births.len(), 3, "{births:?}");
    for birth in &births {
        let BirthObservation::BirthRefused { refusal, .. } = birth else {
            panic!("a bare identity has no birth: {birth:?}");
        };
        assert!(refusal.contains("UnknownName"), "{refusal}");
    }
    let reads = vec![GroupRead::Loop(bare.loops[0].1, NameSelection::Exact)];
    let answers = groups::read(&world, "fa-bare-group", reads);
    assert!(
        format!("{answers:?}").contains("UnknownName"),
        "a bare group identity refuses typed: {answers:?}"
    );
    // The same bytes are genuine where they were born.
    let genuine = served(&elsewhere, &bare, "fb-genuine");
    assert!(
        !genuine.iter().any(|line| line.contains("UnknownName")),
        "{genuine:#?}"
    );
}
