//! A recorded birth's discriminator references decode only as the kind the
//! read admitted them under, relations read and look up their births like
//! entities, and a lookup never matches a birth by a same-commit reference.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorDecode,
    ApplicationNameDiscriminatorDecodeDenial as Denial, ApplicationNameDiscriminatorDecoder,
    ApplicationNameDiscriminatorEncoder, ApplicationNameSubjectKind, EntityNameSubject,
    NameBirthDecodeDenial, Unowned, WorthQueryName, WorthQueryNamedKind,
};
use worth_query_execution_naming::facade::{
    ExecutionNameReadDenial, NameReadCurrentness, NameReadDependencies, WorthQueryOutputBirth,
};

use crate::naming_execution_fixture::{
    execution_lease_for_workers, CadNamingSchema, Edge, Face, FaceSplitDisc, Seam, SeamSplitDisc,
};
use crate::naming_same_commit_names::{side, Authoring};
use crate::naming_tracked_world::TrackedWorld;

type Decoded<T> = Result<T, Denial>;

impl WorthQueryNamedKind for Face {
    type Subject = EntityNameSubject;
    type OwnerScope = Unowned;
}
impl WorthQueryNamedKind for Edge {
    type Subject = EntityNameSubject;
    type OwnerScope = Unowned;
}

/// The fixture's discriminators are written by `worth_query_name_discriminator!`,
/// which encodes only; these read them back by hand.
impl ApplicationNameDiscriminatorDecode for FaceSplitDisc {
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self> {
        match decoder.variant()? {
            "Side" => Ok(Self::Side {
                side: decoder.field("side")?,
            }),
            "Cutter" => Ok(Self::Cutter {
                cutter: decoder.field("cutter")?,
                piece: decoder.field("piece")?,
            }),
            other => Err(decoder.unknown_variant(other)),
        }
    }
}

impl ApplicationNameDiscriminatorDecode for SeamSplitDisc {
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self> {
        match decoder.variant()? {
            "Ordinal" => Ok(Self::Ordinal {
                index: decoder.field("index")?,
            }),
            "ByFace" => Ok(Self::ByFace {
                face: decoder.field("face")?,
                index: decoder.field("index")?,
            }),
            other => Err(decoder.unknown_variant(other)),
        }
    }
}

/// The seam discriminator's identity and wire form, misdeclared over edges.
#[derive(Debug)]
struct SeamByEdge {
    face: WorthQueryName<Edge>,
}
impl ApplicationNameDiscriminator for SeamByEdge {
    const DISCRIMINATOR_IDENTITY: &'static str = SeamSplitDisc::DISCRIMINATOR_IDENTITY;
    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.variant("ByFace").field("face", &self.face);
    }
}
impl ApplicationNameDiscriminatorDecode for SeamByEdge {
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self> {
        decoder.variant()?;
        Ok(Self {
            face: decoder.field("face")?,
        })
    }
}

fn seam_birth(key: &str, face: WorthQueryName<Face>) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::new(key, "seam")
        .with_discriminator(&SeamSplitDisc::ByFace { face, index: 7 })
}

#[test]
fn a_relation_reads_its_birth_and_its_face_reference_decodes_only_as_a_face() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2"]);
    let (f1, f2) = (seeded[0], seeded[1]);
    let before = world.head();
    let (seam, linked) = world.seam(&execution, "occ.seam", f1, f2);

    // Before the link, the lookup awaits the seam's birth; the link stales it.
    let mut awaiting = NameReadDependencies::new();
    let reader = world.reader(&execution, &before);
    let birth = seam_birth("occ.seam", f1.name);
    let lookup = reader.relation_name_for_birth::<Seam>(&birth, &[], &mut awaiting);
    assert_eq!(lookup, Ok(None));
    assert_eq!(
        reader.relation_birth_of::<Seam>(&seam.name, &mut awaiting),
        Err(ExecutionNameReadDenial::UnknownName)
    );
    assert!(matches!(
        awaiting.currentness_after_commit(&linked),
        NameReadCurrentness::Stale(_)
    ));

    let head = world.head();
    let reader = world.reader(&execution, &head);
    let mut deps = NameReadDependencies::new();
    assert_eq!(
        reader.relation_name_for_birth::<Seam>(&birth, &[], &mut deps),
        Ok(Some(seam.name))
    );
    let recorded = reader
        .relation_birth_of::<Seam>(&seam.name, &mut deps)
        .expect("the seam's birth");
    assert!(deps.is_empty(), "visible births rest on nothing");
    assert_eq!(
        (recorded.occurrence_key(), recorded.role_path()),
        ("occ.seam", "seam")
    );
    assert_eq!(
        recorded.decode_discriminator::<SeamSplitDisc>(),
        Ok(Some(SeamSplitDisc::ByFace {
            face: f1.name,
            index: 7
        }))
    );
    // The recorded face is never an edge, whatever the declared field says.
    assert_eq!(
        recorded.decode_discriminator::<SeamByEdge>().map(|_| ()),
        Err(NameBirthDecodeDenial::Discriminator(
            Denial::NameKindMismatch {
                digest: *f1.name.as_bytes(),
                expected: (ApplicationNameSubjectKind::Entity, "Edge".to_owned()),
                found: (ApplicationNameSubjectKind::Entity, "Face".to_owned()),
            }
        ))
    );
}

#[test]
fn a_relation_birth_decodes_the_split_piece_it_references() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2"]);
    let (pieces, _) = world.split(&execution, seeded[0], "occ.cut", &[("a", 1), ("b", 2)]);
    let (seam, _) = world.seam(&execution, "occ.seam", pieces[0], seeded[1]);
    let head = world.head();
    let mut deps = NameReadDependencies::new();
    let recorded = world
        .reader(&execution, &head)
        .relation_birth_of::<Seam>(&seam.name, &mut deps)
        .expect("the seam's birth");
    assert_eq!(
        recorded.decode_discriminator::<SeamSplitDisc>(),
        Ok(Some(SeamSplitDisc::ByFace {
            face: pieces[0].name,
            index: 7
        }))
    );
}

#[test]
fn a_lookup_by_a_same_commit_reference_refuses() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    world.seed(&execution, "occ.seed", &["f1"]);
    let mut authoring = Authoring::new();
    authoring.birth("a", side("occ.patch", "cut_piece", 1));
    let patch = authoring.patch("occ.patch", "a", 1);
    let head = world.head();
    let mut deps = NameReadDependencies::new();
    assert_eq!(
        world
            .reader(&execution, &head)
            .entity_name_for_birth::<CadNamingSchema, Face>(&patch, &[], &mut deps),
        Err(ExecutionNameReadDenial::SameCommitBirthReference)
    );
    assert!(deps.is_empty());
}
