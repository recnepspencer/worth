//! Phase 6 platform fit: a decision reads a held name's recorded birth and
//! decodes it into the scheme's typed role and discriminator, whatever the
//! name's lineage did since, and looks a name up by its birth parts. An
//! anchored occurrence path round-trips through a recorded birth.
use super::*;
use probe::births::{self, BirthObservation};
use worth_query_decl::facade::application_naming::{
    NameBirth, NameBirthDecodeDenial, OccurrencePath, RecordedNameBirth,
};

impl World {
    /// Runs a probe over `keys` and returns its birth observations, with
    /// the debug form of how the run ended (sealed or refused).
    pub(super) fn probe_births(
        &self,
        token: &str,
        keys: &[&str],
        ordinal: u64,
    ) -> (Vec<BirthObservation>, String) {
        let ended = self
            .request()
            .advanced()
            .mutate(probe::Intent {
                token: token.to_owned(),
                keys: self::keys(keys),
                value: 400 + ordinal,
            })
            .without_source()
            .idempotency(&format!("birth-probe:{token}:{ordinal}"))
            .execute_in_program(
                &self.host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        probe::take(token);
        (births::take(token), format!("{ended:?}"))
    }
}

pub(super) fn born<'o>(
    observed: &'o [BirthObservation],
    label: &str,
) -> &'o RecordedNameBirth<Member> {
    observed
        .iter()
        .find_map(|observation| match observation {
            BirthObservation::Born { label: l, birth } if l == label => Some(birth),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{label}` must read its birth: {observed:?}"))
}

pub(super) fn minted(observed: &[BirthObservation], label: &str) -> Option<WorthQueryName<Member>> {
    observed
        .iter()
        .find_map(|observation| match observation {
            BirthObservation::Minted { label: l, name } if l == label => Some(*name),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{label}` must be looked up: {observed:?}"))
}

/// A generated placement anchored to repetition 2 of a base pattern.
fn placement() -> OccurrencePath {
    let base = OccurrencePath::root("base")
        .and_then(|path| path.index("instance", 2))
        .expect("a canonical anchor");
    OccurrencePath::root("generator")
        .and_then(|path| path.anchored("placement", &base))
        .expect("a canonical anchored path")
}

fn of_role(occurrence: &OccurrencePath, role: MemberRole) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::of_role(occurrence, role)
}

#[test]
fn a_birth_decodes_into_its_typed_role_discriminator_and_anchored_occurrence() {
    let token = "births-typed";
    let world = World::install();
    let occurrence = placement();
    world.committed(
        token,
        Plan {
            creates: keys(&["a", "i2", "i10"]),
            steps: vec![
                Step::Birth("a".into(), of_role(&occurrence, MemberRole::Authored)),
                Step::Birth("i2".into(), of_role(&occurrence, MemberRole::Instance(2))),
                // Its cutter is born by the same commit: the recorded birth
                // holds the committed name, which decodes as `Committed`.
                Step::BirthSided {
                    key: "i10".into(),
                    birth: of_role(&occurrence, MemberRole::Instance(10)),
                    side: CutSide::Negative,
                    of: "a".into(),
                    along: None,
                },
            ],
            ..Plan::default()
        },
    );
    let observed = world.probe(token, &["a", "i2", "i10"], 0);
    let [a, i2, i10] = ["a", "i2", "i10"].map(|key| named(&observed, key));
    for (label, name) in [("a", a), ("i2", i2), ("i10", i10)] {
        births::birth_of(token, label, name);
    }
    let (observed, ended) = world.probe_births(token, &[], 1);
    assert!(ended.contains("Committed"), "visible births seal: {ended}");

    let tenth = born(&observed, "i10");
    assert_eq!(tenth.occurrence_key(), occurrence.to_occurrence_key());
    assert_eq!(
        tenth.decode::<MemberRole, SidedCut>(),
        Ok(NameBirth {
            name: i10,
            occurrence: occurrence.clone(),
            role: MemberRole::Instance(10),
            parents: Vec::new(),
            discriminator: Some(SidedCut::By {
                cutter: NameRef::Committed(a),
                side: CutSide::Negative,
                along: None,
            }),
        })
    );
    let second = born(&observed, "i2");
    assert_eq!(
        second.decode_role::<MemberRole>(),
        Ok(MemberRole::Instance(2))
    );
    assert_eq!(second.decode_discriminator::<SidedCut>(), Ok(None));
    let authored = born(&observed, "a");
    assert_eq!(
        authored.decode_role::<MemberRole>(),
        Ok(MemberRole::Authored)
    );
    assert_eq!(authored.occurrence(), Ok(occurrence));
    assert_eq!(authored.decode_discriminator::<SidedCut>(), Ok(None));
    // The wrong discriminator type refuses typed rather than misreading.
    assert!(matches!(
        tenth.decode_discriminator::<MemberCut>(),
        Err(NameBirthDecodeDenial::Discriminator(_))
    ));
}

/// Seeds `body` and `tool`, splits `body` into `left`/`right` cut by `tool`.
fn split_world(token: &str) -> (World, [WorthQueryName<Member>; 4]) {
    let world = World::install();
    world.committed(
        &format!("{token}-seed"),
        Plan {
            creates: keys(&["body", "tool"]),
            steps: vec![
                Step::Birth("body".into(), birth("body", "authored")),
                Step::Birth("tool".into(), birth("tool", "authored")),
            ],
            ..Plan::default()
        },
    );
    let observed = world.probe(token, &["body", "tool"], 0);
    let (body, tool) = (named(&observed, "body"), named(&observed, "tool"));
    world.committed(
        &format!("{token}-split"),
        Plan {
            reads: keys(&["body"]),
            deletes: keys(&["body"]),
            creates: keys(&["left", "right"]),
            steps: vec![Step::Split(
                read("body"),
                vec![
                    ("left".into(), piece("cut-1", 1, Some(tool))),
                    ("right".into(), piece("cut-1", 2, Some(tool))),
                ],
            )],
            ..Plan::default()
        },
    );
    let observed = world.probe(token, &["left", "right"], 1);
    let (left, right) = (named(&observed, "left"), named(&observed, "right"));
    (world, [body, tool, left, right])
}

#[test]
fn split_merged_and_retired_names_keep_their_births_and_name_their_parents() {
    let token = "births-lineage";
    let (world, [body, tool, left, right]) = split_world(token);
    world.committed(
        &format!("{token}-merge"),
        Plan {
            reads: keys(&["left", "right"]),
            deletes: keys(&["right"]),
            steps: vec![Step::Merge(
                vec![read("left"), read("right")],
                Keeper::InPlace(read("left")),
            )],
            ..Plan::default()
        },
    );
    for (label, name) in [("body", body), ("left", left), ("right", right)] {
        births::birth_of(token, label, name);
    }
    let (observed, ended) = world.probe_births(token, &[], 2);
    assert!(ended.contains("Committed"), "{ended}");
    let parent = born(&observed, "body");
    assert_eq!(parent.decode_role::<MemberRole>(), Ok(MemberRole::Authored));
    assert!(parent.parents().is_empty());
    for (label, side) in [("left", 1), ("right", 2)] {
        let piece = born(&observed, label);
        assert_eq!(piece.parents(), &[body], "{label}");
        assert_eq!(piece.decode_role::<MemberRole>(), Ok(MemberRole::Piece));
        assert_eq!(
            piece.decode_discriminator::<MemberCut>(),
            Ok(Some(MemberCut::Side {
                side,
                cutter: Some(tool)
            }))
        );
    }
}

#[test]
fn name_for_birth_finds_recorded_births_and_an_unborn_answer_never_seals() {
    let token = "births-lookup";
    let (world, [body, tool, left, _]) = split_world(token);
    births::name_for_birth(
        token,
        "seeded",
        birth("worth.bootstrap-seed/direct", "authored"),
        Vec::new(),
    );
    births::name_for_birth(token, "left", piece("cut-1", 1, Some(tool)), vec![body]);
    births::name_for_birth(token, "unborn", birth("later", "authored"), Vec::new());
    let (observed, ended) = world.probe_births(token, &[], 2);
    let direct = named(
        &world.probe(&format!("{token}-direct"), &["direct"], 3),
        "direct",
    );
    assert_eq!(minted(&observed, "seeded"), Some(direct));
    assert_eq!(minted(&observed, "left"), Some(left));
    assert_eq!(minted(&observed, "unborn"), None);
    assert!(
        !ended.contains("Committed"),
        "an answer resting on an unborn name never seals: {ended}"
    );

    // Once `later` is born, and `left` retired, both lookups answer and seal.
    world.committed(
        &format!("{token}-later"),
        Plan {
            creates: keys(&["later"]),
            reads: keys(&["left"]),
            deletes: keys(&["left"]),
            steps: vec![
                Step::Birth("later".into(), birth("later", "authored")),
                Step::Retire(read("left")),
            ],
            ..Plan::default()
        },
    );
    let (observed, ended) = world.probe_births(token, &[], 4);
    assert!(ended.contains("Committed"), "{ended}");
    let later = named(
        &world.probe(&format!("{token}-later"), &["later"], 5),
        "later",
    );
    assert_eq!(minted(&observed, "unborn"), Some(later));
    assert_eq!(
        minted(&observed, "left"),
        Some(left),
        "a retired name keeps its birth"
    );
    // The looked-up name resolves to what became of it.
    probe::hold(token, "left", left);
    let observed = world.probe(token, &[], 6);
    assert_eq!(
        resolution(&observed, "left"),
        &probe::Observation::Retired {
            label: "left".into()
        }
    );
}

#[test]
fn a_seam_reads_its_birth_and_is_found_by_its_birth_parts() {
    let token = "births-seam";
    let world = World::install();
    let weld = seam_names::seam("births-weld", "seam", 1);
    world.committed(
        &format!("{token}-link"),
        seam_names::link("s1", "direct", "nested", weld.clone()),
    );
    let s1 = world.seam_name(&format!("{token}-read"), "direct", "nested");
    births::seam_birth_of(token, "s1", s1);
    births::seam_name_for_birth(token, "found", weld);
    births::seam_name_for_birth(token, "unborn", seam_names::seam("births-weld", "seam", 2));
    let (observed, ended) = world.probe_births(token, &[], 0);
    assert!(
        !ended.contains("Committed"),
        "an unborn seam lookup never seals: {ended}"
    );
    let seam_minted = |label: &str| {
        observed.iter().find_map(|observation| match observation {
            BirthObservation::SeamMinted { label: l, name } if l == label => Some(*name),
            _ => None,
        })
    };
    assert_eq!(seam_minted("found"), Some(Some(s1)));
    assert_eq!(seam_minted("unborn"), Some(None));
    let Some(BirthObservation::SeamBorn { birth, .. }) = observed
        .iter()
        .find(|observation| matches!(observation, BirthObservation::SeamBorn { .. }))
    else {
        panic!("the seam's birth is read: {observed:?}");
    };
    assert_eq!(birth.name(), s1);
    assert_eq!(birth.occurrence_key(), "births-weld");
    assert_eq!(birth.decode_role::<SeamRole>(), Ok(SeamRole::Seam));
    assert_eq!(
        birth.decode_discriminator::<SeamSide>(),
        Ok(Some(SeamSide { index: 1 }))
    );
}
