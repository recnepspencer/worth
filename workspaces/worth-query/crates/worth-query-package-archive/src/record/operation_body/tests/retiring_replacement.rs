//! A new retirement posture cannot be silently downgraded to historical retention.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
};
#[test]
fn retiring_replacement_v34_roundtrips_and_older_posture_readers_refuse() {
    let link = Link {
        relation: "Current".into(),
        from: "Row".into(),
        to: "Snapshot".into(),
        key: "current".into(),
        direction: Direction::FromScope,
        posture: Posture::ReplaceAndRetireSingle,
    };
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        Some(
            ApplicationOperationBodyCreation::from_untrusted(
                "Snapshot".into(),
                "snapshot".into(),
                vec![],
            )
            .with_untrusted_links(vec![link.clone()]),
        ),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(2048);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &34_u16.to_be_bytes());
    let budget = |bytes: &[u8]| {
        RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap()
    };
    let mut input = BinaryInput::new(&bytes);
    let mut decoded = budget(&bytes);
    assert_eq!(decode(&mut input, &mut decoded).unwrap(), body);
    assert!(input.is_finished());
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measure, &body).unwrap();
    assert_eq!(measure.nested_entries(), decoded.finish().nested_entries());
    bytes[..2].copy_from_slice(&33_u16.to_be_bytes());
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes))
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
    assert_eq!(
        super::super::creation_links::write(&mut BinaryOutput::with_capacity(256), &[link], 33)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
}
