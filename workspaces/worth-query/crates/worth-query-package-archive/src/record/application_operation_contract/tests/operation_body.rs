//! Whole-operation body extension, preserving the legacy body-free prefix.
use super::{
    encode_untrusted, fixture, WorthQueryPackageArchiveLimits,
    WorthQueryPackageArchiveRecordDecoder, WorthQueryPortableApplicationOperationContractRecord,
};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyProgram,
        ApplicationOperationBodyScopeField,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

#[test]
fn operation_body_contract_extension_roundtrips_without_discarding_values() {
    let mut parts = fixture::complete_parts();
    parts.body = Some(
        ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![
            ApplicationOperationBodyScopeField::Assign {
                field: ApplicationFieldBindingLocus::new("Account", "Profile", "name"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.rust.string".to_owned(),
                ),
                value: AspectValue::String(InternedString::Raw("reviewed".to_owned())),
            },
        ]),
    );
    let record = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(parts);
    let bytes = encode_untrusted(&record, WorthQueryPackageArchiveLimits::DEFAULT);
    let decoded =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT)
            .decode_frame(&bytes)
            .unwrap();
    assert_eq!(decoded.record(), &fixture::wrapped(record));
}
