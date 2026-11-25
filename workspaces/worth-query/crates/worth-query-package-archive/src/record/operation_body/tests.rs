//! Codec evidence only: decoded descriptions still require native readmission.
use super::*;
use crate::{binary_output::BinaryOutput, limits::WorthQueryPackageArchiveLimits};
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
#[path = "tests/children.rs"]
mod children;
#[path = "tests/collection_values.rs"]
mod collection_values;
#[path = "tests/creation_links.rs"]
mod creation_links;
#[path = "tests/creation_reuse.rs"]
mod creation_reuse;
#[path = "tests/enrollment.rs"]
mod enrollment;
#[path = "tests/external_emission.rs"]
mod external_emission;
#[path = "tests/field_assignment_versions.rs"]
mod field_assignment_versions;
#[path = "tests/genesis.rs"]
mod genesis;
#[path = "tests/input_expectation.rs"]
mod input_expectation;
#[path = "tests/optional_input.rs"]
mod optional_input;
#[path = "tests/outputs.rs"]
mod outputs;
#[path = "tests/path_capture.rs"]
mod path_capture;
#[path = "tests/path_input.rs"]
mod path_input;
#[path = "tests/path_value.rs"]
mod path_value;
mod peers;
#[path = "tests/principal.rs"]
mod principal;
#[path = "tests/principal_links.rs"]
mod principal_links;
#[path = "tests/reference_input.rs"]
mod reference_input;
#[path = "tests/related_outputs.rs"]
mod related_outputs;
#[path = "tests/related_source.rs"]
mod related_source;
#[path = "tests/requirements.rs"]
mod requirements;
#[path = "tests/retirements.rs"]
mod retirements;
#[path = "tests/retiring_replacement.rs"]
mod retiring_replacement;
#[path = "tests/self_edge_links.rs"]
mod self_edge_links;
#[path = "tests/signed_evidence.rs"]
mod signed_evidence;
#[path = "tests/signing.rs"]
mod signing;
#[path = "tests/succession.rs"]
mod succession;
#[path = "tests/values.rs"]
mod values;

#[test]
fn operation_body_false_literal_roundtrips_and_unknown_version_is_refused() {
    let body = ApplicationOperationBodyProgram::from_untrusted_scope_fields(
        ARCHIVE_DECODE,
        vec![ApplicationOperationBodyScopeField::Assign {
            field: ApplicationFieldBindingLocus::new("Item", "Facts", "Ready"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "worth.rust.bool".to_owned(),
            ),
            value: AspectValue::Bool(false),
        }],
    );
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
    bytes[..2].copy_from_slice(&u16::MAX.to_be_bytes());
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
fn required_presence_uses_version_six_and_old_reader_refuses_the_opcode() {
    let body = ApplicationOperationBodyProgram::from_untrusted_scope_fields(
        ARCHIVE_DECODE,
        vec![ApplicationOperationBodyScopeField::RequirePresent {
            field: ApplicationFieldBindingLocus::new("Item", "Draft", "Note"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "worth.rust.string".to_owned(),
            ),
            scalar_family: ScalarAspectType::String,
        }],
    );
    let mut output = BinaryOutput::with_capacity(128);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &6_u16.to_be_bytes());
    let mut input = BinaryInput::new(&bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
    bytes[..2].copy_from_slice(&5_u16.to_be_bytes());
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
fn optional_set_and_clear_use_version_five_without_reinterpreting_v1_to_v4() {
    let body = ApplicationOperationBodyProgram::from_untrusted_scope_fields(
        ARCHIVE_DECODE,
        vec![ApplicationOperationBodyScopeField::SetOptionalInput {
            field: ApplicationFieldBindingLocus::new("Item", "Draft", "Note"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "worth.rust.string".to_owned(),
            ),
            scalar_family: ScalarAspectType::String,
            member: "note".to_owned(),
        }],
    );
    let mut output = BinaryOutput::with_capacity(128);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &5_u16.to_be_bytes());
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

    let clear = ApplicationOperationBodyProgram::from_untrusted_scope_fields(
        ARCHIVE_DECODE,
        vec![ApplicationOperationBodyScopeField::ClearOptional {
            field: ApplicationFieldBindingLocus::new("Item", "Draft", "Note"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "worth.rust.string".to_owned(),
            ),
            scalar_family: ScalarAspectType::String,
        }],
    );
    let mut output = BinaryOutput::with_capacity(128);
    write(&mut output, &clear).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &5_u16.to_be_bytes());
    let mut input = BinaryInput::new(&bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(decode(&mut input, &mut budget).unwrap(), clear);
}

#[test]
fn version_five_creation_flag_preserves_recipe_and_rejects_an_invalid_flag() {
    let body = ApplicationOperationBodyProgram::from_untrusted_with_creation(
        ARCHIVE_DECODE,
        vec![ApplicationOperationBodyScopeField::ClearOptional {
            field: ApplicationFieldBindingLocus::new("Item", "Draft", "Note"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "worth.rust.string".to_owned(),
            ),
            scalar_family: ScalarAspectType::String,
        }],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            ARCHIVE_DECODE,
            "Audit".to_owned(),
            "audit".to_owned(),
            vec![ApplicationOperationBodyCreatedField::Literal {
                field: ApplicationFieldBindingLocus::new("Audit", "Facts", "Ready"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted(
                    ARCHIVE_DECODE,
                    "worth.rust.bool".to_owned(),
                ),
                value: AspectValue::Bool(true),
            }],
        )),
    );
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &5_u16.to_be_bytes());
    let mut input = BinaryInput::new(&bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());

    // Locate the framing flag by parsing the one optional scope step.
    let mut prefix = BinaryInput::new(&bytes);
    assert_eq!(prefix.u16().unwrap(), 5);
    assert_eq!(prefix.u32().unwrap(), 1);
    assert_eq!(prefix.u16().unwrap(), 5);
    for _ in 0..4 {
        prefix.text().unwrap();
    }
    assert_eq!(prefix.u16().unwrap(), 16);
    let flag = bytes.len() - prefix.remaining_len();
    assert_eq!(&bytes[flag..flag + 2], &1_u16.to_be_bytes());
    bytes[flag..flag + 2].copy_from_slice(&2_u16.to_be_bytes());
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
