//! Carriage preserves consequence meaning; a prior version cannot consume its new tail.
use super::*;
use std::num::NonZeroUsize;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyOwnedSelectionRetirement as Rule;

#[test]
fn owned_selection_retirement_v31_roundtrips_and_cannot_be_fully_read_as_v30() {
    let binding = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into());
    let trigger = ApplicationFieldBindingLocus::new("Item", "Facts", "Company");
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Item".into(),
        vec![ApplicationOperationBodyScopeField::AssignInput {
            field: trigger.clone(),
            binding: binding.clone(),
            scalar_family: ScalarAspectType::String,
            member: "company".into(),
        }],
        None,
        vec![],
    )
    .with_untrusted_retirements(vec![Rule {
        trigger,
        binding,
        scalar_family: ScalarAspectType::String,
        member: "company".into(),
        owners_relation: "OwnedRows".into(),
        scope: "Item".into(),
        owner: "Row".into(),
        selection_relation: "Selection".into(),
        selected: "Snapshot".into(),
        maximum_owners: NonZeroUsize::new(256).unwrap(),
    }]);
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &31_u16.to_be_bytes());
    let budget = || {
        RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap()
    };
    let mut input = BinaryInput::new(&bytes);
    let mut decoded_work = budget();
    assert_eq!(decode(&mut input, &mut decoded_work).unwrap(), body);
    assert!(input.is_finished());
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measure, &body).unwrap();
    assert_eq!(
        measure.nested_entries(),
        decoded_work.finish().nested_entries()
    );
    bytes[..2].copy_from_slice(&30_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    let downgraded = decode(&mut input, &mut budget).unwrap();
    assert_ne!(downgraded, body);
    assert!(
        !input.is_finished(),
        "the containing record rejects an unconsumed tail"
    );
    bytes[..2].copy_from_slice(&31_u16.to_be_bytes());
    let bound = bytes.len() - 8;
    bytes[bound..].copy_from_slice(&0_u64.to_be_bytes());
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
        Kind::InvalidRecordShape
    );
}
