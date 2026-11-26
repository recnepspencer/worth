//! #426 through a coupled producer, as CAD's body chain regenerates: an
//! authored edit creates and names a member, and the producer the edit
//! demands in the same envelope reads that member's name and birth while it
//! decides. The producer seals and the envelope commits exactly the names
//! it read. A pending birth's discriminator decodes only under the kinds its
//! references were born as, pending or committed.
use super::super::producer::{pending_reads, shape};
use super::*;
use crate::persistent_naming::invariant_names::refusals::{CutAlongMember, CutAlongSeam};
use probe::births::{self, BirthObservation};
use worth_query_decl::facade::application_naming::{
    ApplicationNameDiscriminatorDecodeDenial, NameBirthDecodeDenial,
};

/// The `shape` edit creates and names `fresh`; the producer it demands reads
/// `fresh`'s name and birth on the unpublished candidate, seals, and the
/// envelope commits exactly what it read.
#[test]
fn a_coupled_producer_reads_an_earlier_stage_birth_and_seals() {
    let (token, value) = ("pend-producer", 4261);
    let world = World::install();
    shape::bear(value, "fresh", "pend-producer-fresh");
    pending_reads::read_pending(value, "fresh");
    let outcome = world.shape("a", value, false, token);
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    let read = pending_reads::observed(value);
    let Some(Ok((name, birth))) = read else {
        panic!("the producer read the pending member: {read:?}")
    };
    assert_eq!(birth.name(), name);
    assert!(world.live(&super::super::producer::primary_key("a")));
    let after = format!("{token}-after");
    births::birth_of_member(&after, "fresh", "fresh");
    let committed = world.probe(&after, &["fresh"], 1);
    assert_eq!(outputs::named(&committed, "fresh"), name);
    assert_eq!(
        births::take(&after),
        vec![BirthObservation::Born {
            label: "fresh".into(),
            birth,
        }],
        "the birth the commit records, byte for byte"
    );
}

/// An earlier stage bears member `b` cut along seam `s`, which the same
/// stage links and names: the later stage reads `b`'s pending birth, whose
/// reference to the pending seam decodes as a seam and never as a member.
#[test]
fn a_pending_birth_references_a_pending_name_of_its_own_kind_only() {
    let token = "pend-kind";
    let world = World::install();
    births::birth_of_member(token, "b", "b");
    let outcome = world.parts(
        token,
        vec![
            sculpted(
                "pend-kind-1",
                Plan {
                    creates: keys(&["a1", "b"]),
                    seams: vec![("s".into(), "a1".into(), "b".into())],
                    steps: vec![
                        Step::Birth("a1".into(), birth("pend-kind", "authored")),
                        Step::SeamBirthOf {
                            key: "s".into(),
                            birth: birth("pend-kind", "seam"),
                            index: 1,
                            of: "a1".into(),
                        },
                        Step::BirthAlong {
                            key: "b".into(),
                            birth: birth("pend-kind", "piece"),
                            side: 1,
                            seam: "s".into(),
                        },
                    ],
                    ..Plan::default()
                },
            ),
            probed(token, &["b"]),
        ],
    );
    assert!(matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
    let observed = births::take(token);
    let [BirthObservation::Born { birth, .. }] = observed.as_slice() else {
        panic!("the pending piece's birth reads: {observed:?}")
    };
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
    let after = format!("{token}-after");
    births::birth_of_member(&after, "b", "b");
    world.probe(&after, &["b"], 1);
    assert_eq!(
        births::take(&after),
        observed,
        "the birth its commit records"
    );
}
