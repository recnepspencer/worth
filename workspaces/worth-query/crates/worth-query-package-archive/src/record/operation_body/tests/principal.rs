//! Actor metadata survives transport; transport never supplies admission authority.
use super::*;

#[test]
fn principal_initializer_roundtrips_only_as_version_twelve() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Item".to_owned(),
        vec![],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            "Audit".to_owned(),
            "audit".to_owned(),
            vec![ApplicationOperationBodyCreatedField::PrincipalIdentity {
                field: ApplicationFieldBindingLocus::new("Audit", "Facts", "Actor"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.rust.u64".to_owned(),
                ),
                scalar_family: ScalarAspectType::UInt64,
                mutation_binding: "ChangeBinding".to_owned(),
                principal_binding: "PrincipalBinding".to_owned(),
                source: ApplicationFieldBindingLocus::new("Principal", "Identity", "Id"),
            }],
        )),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &12_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
    bytes[..2].copy_from_slice(&11_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
}
