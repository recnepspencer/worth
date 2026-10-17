use worth_foundational::facade::{ScalarAspectType, ValueDecodeBounds};
use worth_query_declaration::facade::application_schema::{
    ApplicationCanonicalValueContract, ApplicationSchemaMember,
    WorthQueryPortableApplicationSchemaParts, WorthQueryPortableApplicationSchemaRecord,
};

use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
use crate::limits::WorthQueryPackageArchiveLimits;

#[test]
fn unsupported_canonical_metadata_is_refused_before_payload_or_member_bytes() {
    let source = super::fixture::complete_untrusted_schema_record();
    let mut field = source.members()[2].clone();
    let ApplicationSchemaMember::Field {
        canonical,
        scalar_family,
        ..
    } = &mut field
    else {
        panic!("fixture field must be present")
    };
    *canonical = Some(ApplicationCanonicalValueContract::of::<String>(
        ValueDecodeBounds::new(2048, 128, 8, 12),
    ));
    *scalar_family = ScalarAspectType::String;
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: source.owner().to_owned(),
            name: source.name().to_owned(),
            major: source.major(),
            minor: source.minor(),
            members: vec![source.members()[0].clone(), field.clone()],
            contributions: Vec::new(),
        },
    );
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    assert_eq!(
        super::super::payload_byte_length(&record, limits)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedCanonicalValueContract
    );

    let mut output = BinaryOutput::with_capacity(0);
    let denial = super::super::write_payload(&record, &mut output, limits).unwrap_err();
    assert_eq!(denial.kind(), Kind::UnsupportedCanonicalValueContract);
    assert!(output.into_bytes().is_empty());

    let mut output = BinaryOutput::with_capacity(0);
    let denial = super::super::member::write(&mut output, &field).unwrap_err();
    assert_eq!(denial.kind(), Kind::UnsupportedCanonicalValueContract);
    assert!(output.into_bytes().is_empty());
}
