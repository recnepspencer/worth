use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenialKind;
use crate::limits::WorthQueryPackageArchiveLimits;
use crate::record::decode_budget::RecordDecodeAttempt;

#[test]
fn historical_fact_budget_member_is_refused_without_erasing_its_meaning() {
    // Historical tag 15, operation "Apply", maximum nine: independently frozen
    // valid old grammar rather than bytes emitted by the successor encoder.
    let bytes = [
        0, 15, 0, 0, 0, 5, b'A', b'p', b'p', b'l', b'y', 0, 0, 0, 0, 0, 0, 0, 9,
    ];
    let mut input = BinaryInput::new(&bytes);
    let mut attempt = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let denial = super::super::member::decode(&mut input, &mut attempt).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );
}
