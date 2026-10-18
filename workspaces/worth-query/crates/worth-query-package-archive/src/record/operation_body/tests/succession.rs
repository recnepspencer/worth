//! Transport preserves succession meaning but confers no completed-source authority.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccession;

#[test]
fn succession_v21_roundtrips_and_cannot_be_silently_downgraded() {
    let plain = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Root".into(),
        vec![],
        None,
        vec![],
    );
    let body = plain.clone().with_untrusted_succession(Some(
        ApplicationOperationBodySuccession::from_untrusted(
            "workflow".into(),
            "binding".into(),
            "root".into(),
        ),
    ));
    assert_ne!(body, plain);
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &21_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measure, &body).unwrap();
    assert_eq!(measure.nested_entries(), budget.finish().nested_entries());
    bytes[..2].copy_from_slice(&20_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert!(decode(&mut input, &mut budget)
        .unwrap()
        .succession()
        .is_none());
    assert!(
        !input.is_finished(),
        "v20 cannot consume the v21 attachment"
    );
    for bytes in [vec![2], vec![1], vec![1, 0, 0]] {
        assert!(super::super::succession::decode(&mut BinaryInput::new(&bytes)).is_err());
    }
}
