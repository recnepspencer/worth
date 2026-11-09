//! Codec descriptions preserve the complete flat authored correction contract.
use super::*;
use crate::{
    binary_output::BinaryOutput,
    limits::WorthQueryPackageArchiveLimits,
    record::decode_budget::{RecordDecodeAttempt, WorthQueryPackageArchiveDecodeWork},
};
use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::{
    application_schema::ApplicationFieldPresence, portable_identity::WorthQueryPortableTypeIdentity,
};
fn identity(entity: &str) -> DeclaredAuthoredCorrection {
    DeclaredAuthoredCorrection::from_untrusted_parts(DeclaredAuthoredCorrectionParts {
        family: WorthQueryPortableTypeIdentity::from_untrusted(format!("fixture.{entity}.v1")),
        logical_key: Box::new(ApplicationSchemaMember::Field {
            entity: entity.into(),
            aspect: "Facts".into(),
            field: "Key".into(),
            presence: ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::UInt64,
            value_type: "worth.rust.u64".into(),
            canonical: None,
            content_format: None,
            unit: None,
            frame: None,
            writable: false,
            equality_queryable: false,
            unique: false,
        }),
        binding_unit: None,
        source_role: format!("retired-{entity}"),
        inverse_role: format!("restored-{entity}"),
        source_posture: Posture::Retire,
    })
}
fn roundtrip(value: &DeclaredAuthoredCorrection) -> Result<DeclaredAuthoredCorrection, Denial> {
    let mut output = BinaryOutput::with_capacity(512);
    write(&mut output, value)?;
    let bytes = output.into_bytes();
    let mut budget = RecordDecodeAttempt::begin(
        WorthQueryPackageArchiveDecodeWork::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )?;
    let mut input = BinaryInput::new(&bytes);
    let decoded = decode(&mut input, &mut budget)?;
    assert_eq!(input.remaining_len(), 0);
    Ok(decoded)
}
#[test]
fn authored_member_ownership_and_optional_coanchors_survive_the_archive() {
    let member =
        identity("Member").with_member_owner(PortablePreImageRelationLocus::from_untrusted_fields(
            "MemberOf".into(),
            "Member".into(),
            "Anchor".into(),
        ));
    let value = identity("Anchor")
        .with_co_anchor(member, false)
        .with_co_anchor(identity("Companion"), true);
    assert_eq!(roundtrip(&value).unwrap(), value);
}
#[test]
fn nested_authored_coanchors_refuse_instead_of_opening_a_recursive_wire_shape() {
    let value = identity("Anchor").with_co_anchor(
        identity("Member").with_co_anchor(identity("Nested"), false),
        false,
    );
    assert!(roundtrip(&value).is_err());
}
