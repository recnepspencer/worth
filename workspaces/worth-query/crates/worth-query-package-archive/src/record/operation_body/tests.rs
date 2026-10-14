//! Codec evidence only: decoded descriptions still require native readmission.
use super::*;
use crate::{binary_output::BinaryOutput, limits::WorthQueryPackageArchiveLimits};
use worth_foundational::facade::{AspectValue, ScalarAspectType};

#[test]
fn operation_body_false_literal_roundtrips_and_unknown_version_is_refused() {
    let body = ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![
        ApplicationOperationBodyScopeField::Assign {
            field: ApplicationFieldBindingLocus::new("Item", "Facts", "Ready"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.bool".to_owned()),
            value: AspectValue::Bool(false),
        },
    ]);
    let mut output = BinaryOutput::with_capacity(128);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    // Value-family tag 2, then boolean payload 0; neither is an opcode or permit.
    assert_eq!(&bytes[bytes.len() - 4..], &[0, 2, 0, 0]);
    let mut input = BinaryInput::new(&bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
    bytes[..2].copy_from_slice(&4_u16.to_be_bytes());
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

#[test]
fn created_entity_fields_use_version_three_and_roundtrip_without_authority() {
    let body = ApplicationOperationBodyProgram::from_untrusted_with_creation(
        vec![ApplicationOperationBodyScopeField::Assign {
            field: ApplicationFieldBindingLocus::new("Item", "Facts", "Ready"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.bool".to_owned()),
            value: AspectValue::Bool(true),
        }],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            "Audit".to_owned(),
            "audit-slot".to_owned(),
            vec![
                ApplicationOperationBodyCreatedField::Literal {
                    field: ApplicationFieldBindingLocus::new("Audit", "Facts", "Ready"),
                    binding: WorthQueryPortableTypeIdentity::from_untrusted(
                        "worth.rust.bool".to_owned(),
                    ),
                    value: AspectValue::Bool(true),
                },
                ApplicationOperationBodyCreatedField::Input {
                    field: ApplicationFieldBindingLocus::new("Audit", "Facts", "Note"),
                    binding: WorthQueryPortableTypeIdentity::from_untrusted(
                        "worth.rust.string".to_owned(),
                    ),
                    scalar_family: ScalarAspectType::String,
                    member: "note".to_owned(),
                },
            ],
        )),
    );
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &3_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
}

#[test]
fn input_member_assignment_uses_version_two_and_old_record_refuses_its_opcode() {
    let body = ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![
        ApplicationOperationBodyScopeField::AssignInput {
            field: ApplicationFieldBindingLocus::new("Item", "Facts", "Retention"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".to_owned()),
            scalar_family: ScalarAspectType::UInt64,
            member: "retention_days".to_owned(),
        },
    ]);
    let mut output = BinaryOutput::with_capacity(128);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &2_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());

    bytes[..2].copy_from_slice(&1_u16.to_be_bytes());
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
