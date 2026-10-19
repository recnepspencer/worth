//! Schema-member carriage only; this untrusted codec fixture grants no authority.
use super::{
    fixture, BinaryOutput, RecordDecodeAttempt, WorthQueryPackageArchiveLimits,
    WorthQueryPortablePackageRecord,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyProgram,
        ApplicationOperationBodyScopeField, ApplicationSchemaMember,
        WorthQueryPortableApplicationSchemaRecord,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

#[test]
fn operation_body_schema_member_roundtrips_exact_guard_and_assignment() {
    let mut parts = fixture::complete_untrusted_schema_record().into_parts();
    let field = ApplicationFieldBindingLocus::new("Entity", "Aspect", "field");
    let binding = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".to_owned());
    parts.members.push(ApplicationSchemaMember::OperationBody {
        operation: "Apply".to_owned(),
        body: ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![
            ApplicationOperationBodyScopeField::Expect {
                field: field.clone(),
                binding: binding.clone(),
                value: AspectValue::UInt64(7),
            },
            ApplicationOperationBodyScopeField::Assign {
                field,
                binding,
                value: AspectValue::UInt64(8),
            },
        ]),
    });
    parts.members.sort();
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut output = BinaryOutput::with_capacity(4096);
    super::super::write_payload(&record, &mut output, limits).unwrap();
    let bytes = output.into_bytes();
    let mut input = crate::binary_input::BinaryInput::new(&bytes);
    let mut budget =
        RecordDecodeAttempt::begin(Default::default(), bytes.len() as u64, limits).unwrap();
    let WorthQueryPortablePackageRecord::ApplicationSchema(decoded) =
        super::super::decode_payload(&mut input, &mut budget).unwrap()
    else {
        panic!("schema frame");
    };
    assert_eq!(decoded, record);
    assert!(input.is_finished());
}
