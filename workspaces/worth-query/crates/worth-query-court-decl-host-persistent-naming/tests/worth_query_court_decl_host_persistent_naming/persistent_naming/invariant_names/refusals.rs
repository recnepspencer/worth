//! What an invariant's name reads refuse, and what refuses its commit: a
//! verdict on a decoded pending discriminator, an unnamed kind, a reference
//! of another kind, and a naming plan Relational refuses, which refuses the
//! commit with its exact naming denial whatever the rule decided.
use super::courts::evaluated;
use super::*;
use crate::persistent_naming::outputs::sculpt::Step;
use crate::persistent_naming::outputs::{birth, keys, naming_refusal, piece, read};
use worth_query_decl::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorDecodeDenial, NameBirthDecodeDenial,
};
use worth_query_host::facade::application_naming::LineageNameDenial;

/// The wire form of `CutAlong`, misdeclaring its seam as a member.
#[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.cut-along.v1")]
pub struct CutAlongMember {
    pub side: u64,
    pub seam: WorthQueryName<Member>,
}

/// The wire form of `CutAlong`, as a committed reader decodes it.
#[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.cut-along.v1")]
pub struct CutAlongSeam {
    pub side: u64,
    pub seam: WorthQueryName<Seam>,
}

#[test]
fn a_verdict_on_a_decoded_pending_discriminator_refuses_with_no_effect() {
    let world = WatchWorld::install();
    let before = world.names("watch-forbid-before", &["direct"], 0);
    world.forbid_side(9);
    world.take();
    let outcome = world.sculpt(
        "watch-forbidden",
        Plan {
            creates: keys(&["bad"]),
            steps: vec![Step::Birth("bad".into(), piece("bad", 9, None))],
            ..Plan::default()
        },
    );
    assert_eq!(
        declared_refusal(&format!("{outcome:?}")),
        r#"{"ForbiddenSide":{"key":"bad","side":9}}"#,
        "the watch's verdict refuses the commit: {outcome:?}"
    );
    assert!(
        naming_refusal(&outcome).is_none(),
        "the naming plan was valid"
    );
    assert!(!world.live("bad"), "the refused commit left no record");
    assert_eq!(world.names("watch-forbid-after", &["direct"], 1), before);
    world.committed(
        "watch-allowed",
        Plan {
            creates: keys(&["good"]),
            steps: vec![Step::Birth("good".into(), piece("good", 8, None))],
            ..Plan::default()
        },
    );
    assert!(world.live("good"));
}

#[test]
fn an_unnamed_kind_refuses_typed() {
    let world = WatchWorld::install();
    world.take();
    world.edit_model("b", 4, "watch-unnamed-model");
    let seen = world.take();
    let model = seen
        .iter()
        .find_map(|seen| match seen {
            Seen::Model { key, name } if key == "b" => Some(name.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the watch reads model `b`: {seen:?}"));
    assert!(model.contains("RecordNotNamed"), "{model}");
}

#[test]
fn a_pending_reference_of_another_kind_refuses_name_kind_mismatch() {
    let world = WatchWorld::install();
    world.committed(
        "watch-along",
        Plan {
            creates: keys(&["a", "b"]),
            seams: vec![("s".into(), "a".into(), "b".into())],
            steps: vec![
                Step::Birth("a".into(), birth("along", "authored")),
                Step::SeamBirthOf {
                    key: "s".into(),
                    birth: birth("along", "seam"),
                    index: 1,
                    of: "a".into(),
                },
                Step::BirthAlong {
                    key: "b".into(),
                    birth: birth("along", "piece"),
                    side: 1,
                    seam: "s".into(),
                },
            ],
            ..Plan::default()
        },
    );
    let watched = evaluated(&world.take(), "b");
    let Some(Ok(birth)) = &watched.birth else {
        panic!("the pending piece's birth reads: {:?}", watched.birth)
    };
    // The seam is born by the same commit: the proposed view admits it as
    // the seam it is, so it decodes as one and never as a member.
    assert!(matches!(
        birth.decode_discriminator::<CutAlongSeam>(),
        Ok(Some(CutAlongSeam { side: 1, .. }))
    ));
    assert!(
        matches!(
            birth.decode_discriminator::<CutAlongMember>(),
            Err(NameBirthDecodeDenial::Discriminator(
                ApplicationNameDiscriminatorDecodeDenial::NameKindMismatch { .. }
            ))
        ),
        "{:?}",
        birth.decode_discriminator::<CutAlongMember>()
    );
}

#[test]
fn a_missing_birth_refuses_the_commit_with_its_exact_naming_denial() {
    let world = WatchWorld::install();
    let outcome = world.sculpt(
        "watch-missing",
        Plan {
            creates: keys(&["bare"]),
            ..Plan::default()
        },
    );
    assert!(
        matches!(
            naming_refusal(&outcome),
            Some(LineageNameDenial::MissingBirth { .. })
        ),
        "{outcome:?}"
    );
    let watched = evaluated(&world.take(), "bare");
    match &watched.proposed {
        Err(read) => assert!(read.contains("MissingBirth"), "{read}"),
        other => panic!("the read refuses with the plan's denial: {other:?}"),
    }
    assert!(!world.live("bare"));
}

#[test]
fn a_duplicate_birth_refuses_the_commit_with_its_exact_naming_denial() {
    let world = WatchWorld::install();
    world.committed("watch-dup-seed", authored(&["s"]));
    world.take();
    let outcome = world.sculpt(
        "watch-dup",
        Plan {
            reads: keys(&["s"]),
            deletes: keys(&["s"]),
            creates: keys(&["s1", "s2"]),
            steps: vec![Step::Split(
                read("s"),
                vec![
                    ("s1".into(), piece("cut", 1, None)),
                    ("s2".into(), piece("cut", 1, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    match naming_refusal(&outcome) {
        Some(LineageNameDenial::DuplicateBirth { first, second, .. }) => assert_ne!(first, second),
        other => panic!("colliding siblings refuse DuplicateBirth: {other:?} / {outcome:?}"),
    }
    assert!(world.live("s") && !world.live("s1") && !world.live("s2"));
}

/// The declared refusal bytes a custom invariant violation carries.
fn declared_refusal(rendered: &str) -> String {
    let start = rendered.find("bytes: [").expect("a declared refusal") + "bytes: [".len();
    let end = start + rendered[start..].find(']').expect("closed bytes");
    let bytes = rendered[start..end]
        .split(", ")
        .map(|byte| byte.parse::<u8>().expect("a byte"))
        .collect();
    String::from_utf8(bytes).expect("canonical text")
}
