//! A decoded name reference is exactly the kind the read admitted it as,
//! and an index-only discriminator decodes only when it carries its index.

use super::birth_fixture::*;
use super::{
    AdmittedNameReferences, ApplicationNameDiscriminator,
    ApplicationNameDiscriminatorDecodeDenial as Denial, ApplicationNameDiscriminatorEncoder,
    ApplicationNameSubjectKind as Subject, IndexedRoleDiscriminator, NameBirthDecodeDenial,
    NameRef, NAME_ROLE_INDEX_IDENTITY,
};

fn admitted_as(byte: u8, kind: Subject, subject: &str) -> AdmittedNameReferences {
    AdmittedNameReferences::from([([byte; 32], (kind, subject.to_owned()))])
}

fn mismatch(byte: u8, expected: (Subject, &str), found: (Subject, &str)) -> NameBirthDecodeDenial {
    NameBirthDecodeDenial::Discriminator(Denial::NameKindMismatch {
        digest: [byte; 32],
        expected: (expected.0, expected.1.to_owned()),
        found: (found.0, found.1.to_owned()),
    })
}

#[test]
fn a_face_reference_never_decodes_as_an_edge() {
    let cut = Cut::Sided {
        cutter: name(5).into(),
        side: 1,
    };
    let recorded = birth("Authored", cut.canonical_bytes(), &[5]);
    assert_eq!(recorded.decode_discriminator::<Cut>(), Ok(Some(cut)));
    assert_eq!(
        recorded.decode_discriminator::<EdgeCut>(),
        Err(mismatch(
            5,
            (Subject::Entity, "Edge"),
            (Subject::Entity, "Face")
        ))
    );
}

#[test]
fn a_relation_or_group_reference_never_decodes_as_an_entity() {
    let bytes = Cut::Sided {
        cutter: name(5).into(),
        side: 1,
    }
    .canonical_bytes();
    for (kind, subject) in [(Subject::Relation, "Face"), (Subject::Group, "Face")] {
        let recorded = birth_admitting("Authored", bytes.clone(), admitted_as(5, kind, subject));
        assert_eq!(
            recorded.decode_discriminator::<Cut>(),
            Err(mismatch(5, (Subject::Entity, "Face"), (kind, subject))),
            "{kind:?}"
        );
    }
    // An edge reference decodes as an edge and refuses as a face.
    let edge_bytes = EdgeCut {
        cutter: NameRef::Committed(super::WorthQueryName::from_admitted_digest([6; 32])),
        side: 2,
    }
    .canonical_bytes();
    let edges = birth_admitting(
        "Authored",
        edge_bytes,
        admitted_as(6, Subject::Entity, "Edge"),
    );
    assert!(matches!(
        edges.decode_discriminator::<EdgeCut>(),
        Ok(Some(EdgeCut { side: 2, .. }))
    ));
    assert_eq!(
        edges.decode_discriminator::<Cut>(),
        Err(mismatch(
            6,
            (Subject::Entity, "Face"),
            (Subject::Entity, "Edge")
        ))
    );
}

#[test]
fn an_index_only_birth_is_a_mismatch_for_a_discriminator_after_the_index() {
    let alone = birth(
        "Instance",
        IndexedRoleDiscriminator::index_alone(3).canonical_bytes(),
        &[],
    );
    assert_eq!(alone.decode_discriminator::<Side>(), Ok(None));
    assert_eq!(
        alone.decode_discriminator::<IndexedRoleDiscriminator<Side>>(),
        Err(Denial::IdentityMismatch {
            expected: "test.side.v1".into(),
            found: NAME_ROLE_INDEX_IDENTITY.into(),
        }
        .into())
    );
    assert_eq!(
        alone.decode_discriminator::<IndexedRoleDiscriminator>(),
        Ok(Some(IndexedRoleDiscriminator::index_alone(3)))
    );
}

#[test]
fn an_index_only_discriminator_without_its_index_is_refused() {
    let header_only = ApplicationNameDiscriminatorEncoder::new(NAME_ROLE_INDEX_IDENTITY).finish();
    let offset = header_only.len();
    let malformed = birth("Instance", header_only, &[]);
    let truncated = || NameBirthDecodeDenial::from(Denial::Truncated { offset });
    assert_eq!(malformed.decode_discriminator::<Side>(), Err(truncated()));
    assert_eq!(malformed.role_index(), Err(truncated()));
    assert_eq!(malformed.decode_role::<Role>(), Err(truncated()));
    assert_eq!(
        malformed.decode_discriminator::<IndexedRoleDiscriminator>(),
        Err(truncated())
    );

    // Another field where the index belongs is refused, never skipped.
    let mut other = ApplicationNameDiscriminatorEncoder::new(NAME_ROLE_INDEX_IDENTITY);
    other.field("side", &1_u8);
    let other = birth("Instance", other.finish(), &[]);
    assert_eq!(
        other.decode_discriminator::<Side>(),
        Err(Denial::UnexpectedField {
            expected: "worth.role.index".into(),
            found: "side".into()
        }
        .into())
    );
}
