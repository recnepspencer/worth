//! Stable, authority-free archive descriptions for evolving application programs.

use worth_query_host::facade::declaration::application_program::{
    ApplicationFeatureSpec, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity,
};
use worth_query_package_archive::facade::{
    decode_application_program_description, encode_application_program_description,
    WorthQueryApplicationProgramArchiveCompatibility, WorthQueryPackageArchiveDenialKind,
    WorthQueryPackageArchiveLimits,
};

use crate::bounded_dimension_model::programs::{DimensionProgramP0, DimensionProgramP1};
use crate::bounded_dimension_model::schema::BoundedDimensionSchema;

struct DimensionProgramP0IdentityWithP1Meaning;

impl ApplicationProgramDefinition<BoundedDimensionSchema>
    for DimensionProgramP0IdentityWithP1Meaning
{
    type Contributions =
        <DimensionProgramP1 as ApplicationProgramDefinition<BoundedDimensionSchema>>::Contributions;
    type Outputs =
        <DimensionProgramP1 as ApplicationProgramDefinition<BoundedDimensionSchema>>::Outputs;
    type Rules =
        <DimensionProgramP1 as ApplicationProgramDefinition<BoundedDimensionSchema>>::Rules;

    const IDENTITY: ApplicationProgramIdentity =
        <DimensionProgramP0 as ApplicationProgramDefinition<BoundedDimensionSchema>>::IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        DimensionProgramP1::feature_specs()
    }
}

#[test]
fn validated_p0_and_p1_have_stable_distinct_bounded_archive_descriptions() {
    let p0 = ApplicationProgramAuthoring::<BoundedDimensionSchema, DimensionProgramP0>::begin()
        .validated_program()
        .expect("P0 validates");
    let p1 = ApplicationProgramAuthoring::<BoundedDimensionSchema, DimensionProgramP1>::begin()
        .validated_program()
        .expect("P1 validates");
    let p0_changed = ApplicationProgramAuthoring::<
        BoundedDimensionSchema,
        DimensionProgramP0IdentityWithP1Meaning,
    >::begin()
    .validated_program()
    .expect("changed P0 validates");
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;

    let p0_bytes = encode_application_program_description(&p0, limits).expect("P0 encodes");
    let repeated = encode_application_program_description(&p0, limits).expect("P0 re-encodes");
    let p1_bytes = encode_application_program_description(&p1, limits).expect("P1 encodes");
    let p0_changed_bytes =
        encode_application_program_description(&p0_changed, limits).expect("changed P0 encodes");
    assert_eq!(p0_bytes, repeated, "unchanged meaning keeps exact bytes");
    assert_ne!(p0_bytes, p1_bytes, "changed rule meaning changes bytes");

    let decoded_p0 = decode_application_program_description(&p0_bytes, limits).expect("P0 decodes");
    let decoded_p1 = decode_application_program_description(&p1_bytes, limits).expect("P1 decodes");
    let decoded_p0_changed = decode_application_program_description(&p0_changed_bytes, limits)
        .expect("changed P0 decodes");
    assert_eq!(decoded_p0.identity(), p0.identity().as_str());
    assert_eq!(decoded_p0.revision(), p0.revision().to_string());
    assert_eq!(
        decoded_p0.compatibility_with_validated(&p0),
        WorthQueryApplicationProgramArchiveCompatibility::ExactRevision
    );
    assert_eq!(
        decoded_p0.compatibility_with(&decoded_p1),
        WorthQueryApplicationProgramArchiveCompatibility::DifferentProgram
    );
    assert_eq!(
        decoded_p0.compatibility_with(&decoded_p0_changed),
        WorthQueryApplicationProgramArchiveCompatibility::ChangedRevision
    );

    let mut tampered_bytes = p0_bytes.clone();
    *tampered_bytes
        .last_mut()
        .expect("P0 description is nonempty") ^= 1;
    let tampered = decode_application_program_description(&tampered_bytes, limits)
        .expect("descriptive tamper remains structurally decodable");
    assert_eq!(
        tampered.compatibility_with_validated(&p0),
        WorthQueryApplicationProgramArchiveCompatibility::ChangedRevision,
        "decoded description cannot claim the validated revision with changed facts"
    );
}

#[test]
fn program_description_decode_is_bounded_and_rejects_trailing_mutation() {
    let p0 = ApplicationProgramAuthoring::<BoundedDimensionSchema, DimensionProgramP0>::begin()
        .validated_program()
        .expect("P0 validates");
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut bytes = encode_application_program_description(&p0, limits).expect("P0 encodes");
    bytes.push(0);
    let denial = decode_application_program_description(&bytes, limits)
        .expect_err("trailing bytes never become descriptive meaning");
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::TrailingBytes
    );

    let narrow = WorthQueryPackageArchiveLimits::new(4_096, 1, 8, 8).with_maximum_archive_bytes(8);
    let denial = encode_application_program_description(&p0, narrow)
        .expect_err("the caller's byte ceiling is enforced before allocation");
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::LogicalByteBudgetExceeded
    );
}
