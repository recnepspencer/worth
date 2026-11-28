use worth_foundational::scan_name_discriminator_references;

use super::birth_fixture::*;
use super::{
    AdmittedNameReferences, ApplicationNameDiscriminator,
    ApplicationNameDiscriminatorDecodeDenial as Denial, IndexedRoleDiscriminator,
    NameBirthDecodeDenial, OccurrencePath, RecordedNameBirth,
};

#[test]
fn every_closed_value_reads_back_exactly_as_encoded() {
    let (bytes, references) = every().canonical_bytes_with_references();
    assert_eq!(
        scan_name_discriminator_references(&bytes),
        Ok(references.clone())
    );
    assert_eq!(references, vec![[1; 32], [2; 32]]);
    let recorded = birth("Authored", bytes, &[1, 2]);
    assert_eq!(recorded.decode_discriminator::<Every>(), Ok(Some(every())));
    let decoded = recorded
        .decode::<Role, Every>()
        .expect("the whole birth decodes");
    assert_eq!(decoded.role, Role::Authored);
    assert_eq!(
        decoded.occurrence,
        OccurrencePath::parse("feature/instance[3]").unwrap()
    );
    assert_eq!(decoded.parents, vec![name(0xaa)]);
    assert_eq!(decoded.name, name(0xee));
}

#[test]
fn a_name_reference_the_read_did_not_admit_is_refused() {
    let bytes = every().canonical_bytes();
    assert_eq!(
        birth("Authored", bytes, &[1]).decode_discriminator::<Every>(),
        Err(NameBirthDecodeDenial::Discriminator(
            Denial::UnrecordedName { digest: [2; 32] }
        ))
    );
}

#[test]
fn every_truncation_and_any_trailing_byte_is_refused() {
    let bytes = every().canonical_bytes();
    for len in 1..bytes.len() {
        let decoded = birth("Authored", bytes[..len].to_vec(), &[1, 2]);
        assert!(decoded.decode_discriminator::<Every>().is_err(), "{len}");
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        birth("Authored", trailing, &[1, 2]).decode_discriminator::<Every>(),
        Err(Denial::TrailingBytes {
            offset: bytes.len()
        }
        .into())
    );
}

#[test]
fn another_identity_variant_or_field_is_refused() {
    let side = Side { side: 1 }.canonical_bytes();
    assert_eq!(
        birth("Authored", side, &[]).decode_discriminator::<Cut>(),
        Err(Denial::IdentityMismatch {
            expected: "test.cut.v1".into(),
            found: "test.side.v1".into()
        }
        .into())
    );
    let unknown = CutImpostor::Unknown.canonical_bytes();
    assert_eq!(
        birth("Authored", unknown, &[]).decode_discriminator::<Cut>(),
        Err(Denial::UnknownVariant {
            variant: "Unknown".into()
        }
        .into())
    );
    let renamed = SideImpostor { other: 1 }.canonical_bytes();
    assert_eq!(
        birth("Authored", renamed, &[]).decode_discriminator::<Side>(),
        Err(Denial::UnexpectedField {
            expected: "side".into(),
            found: "other".into()
        }
        .into())
    );
    let sided = Cut::Sided {
        cutter: name(5).into(),
        side: 2,
    };
    let recorded = birth("Authored", sided.canonical_bytes(), &[5]);
    assert_eq!(recorded.decode_discriminator::<Cut>(), Ok(Some(sided)));
    let seam = birth("Authored", Cut::Seam { index: 4 }.canonical_bytes(), &[]);
    assert_eq!(
        seam.decode_discriminator::<Cut>(),
        Ok(Some(Cut::Seam { index: 4 }))
    );
    let not_a_discriminator = birth("Authored", vec![1, 2, 3], &[]);
    assert_eq!(
        not_a_discriminator.decode_discriminator::<Cut>(),
        Err(Denial::NotADiscriminator.into())
    );
}

#[test]
fn an_indexed_role_decodes_its_index_into_the_role() {
    let indexed = IndexedRoleDiscriminator::new(3, Side { side: 1 }).canonical_bytes();
    let recorded = birth("Instance", indexed, &[]);
    assert_eq!(recorded.role_index(), Ok(Some(3)));
    assert_eq!(recorded.decode_role::<Role>(), Ok(Role::Instance(3)));
    assert_eq!(
        recorded.decode_discriminator::<Side>(),
        Ok(Some(Side { side: 1 }))
    );
    let whole = recorded.decode_discriminator::<IndexedRoleDiscriminator<Side>>();
    assert_eq!(
        whole,
        Ok(Some(IndexedRoleDiscriminator::new(3, Side { side: 1 })))
    );

    let alone = birth(
        "Instance",
        IndexedRoleDiscriminator::index_alone(10).canonical_bytes(),
        &[],
    );
    assert_eq!(alone.decode_role::<Role>(), Ok(Role::Instance(10)));
    assert_eq!(alone.decode_discriminator::<Side>(), Ok(None));

    let plain = birth("Authored", Vec::new(), &[]);
    assert_eq!(plain.decode_role::<Role>(), Ok(Role::Authored));
    assert_eq!(plain.decode_discriminator::<Side>(), Ok(None));
    assert_eq!(
        birth("Instance", Vec::new(), &[]).decode_role::<Role>(),
        Err(NameBirthDecodeDenial::UnknownRole {
            role_path: "Instance".into(),
            index: None
        })
    );
    assert_eq!(
        birth("Gone", Vec::new(), &[]).decode_role::<Role>(),
        Err(NameBirthDecodeDenial::UnknownRole {
            role_path: "Gone".into(),
            index: None
        })
    );
}

#[test]
fn an_occurrence_no_path_renders_refuses_typed() {
    let recorded = RecordedNameBirth::<Face>::from_admitted_parts(
        name(1),
        "bare key".into(),
        "Authored".into(),
        Vec::new(),
        Vec::new(),
        AdmittedNameReferences::new(),
    );
    assert!(matches!(
        recorded.decode::<Role, Side>(),
        Err(NameBirthDecodeDenial::Occurrence(_))
    ));
    assert_eq!(recorded.occurrence_key(), "bare key");
}
