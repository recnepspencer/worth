//! V11 carries creation linkage without reinterpreting older body bytes.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLink,
    ApplicationOperationBodyCreationLinkDirection as Direction,
};

#[test]
fn replacement_v19_roundtrips_its_distinct_posture_and_nested_accounting() {
    use worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkPosture as Posture;
    let creation =
        ApplicationOperationBodyCreation::from_untrusted("Item".into(), "item".into(), vec![])
            .with_untrusted_links(vec![ApplicationOperationBodyCreationLink {
                relation: "Current".into(),
                from: "Root".into(),
                to: "Item".into(),
                key: "current".into(),
                direction: Direction::FromScope,
                posture: Posture::ReplaceSingle,
            }]);
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Root".into(),
        vec![],
        Some(creation),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &19_u16.to_be_bytes());
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
}
#[test]
fn creation_link_directions_roundtrip_and_old_record_cannot_consume_them() {
    for direction in [Direction::FromScope, Direction::ToScope] {
        let (from, to) = if direction == Direction::FromScope {
            ("Root", "Item")
        } else {
            ("Item", "Root")
        };
        let creation = ApplicationOperationBodyCreation::from_untrusted(
            "Item".to_owned(),
            "item".to_owned(),
            vec![],
        )
        .with_untrusted_links(vec![ApplicationOperationBodyCreationLink {
            relation: "Ownership".to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
            key: "ownership".to_owned(),
            direction,
            posture: worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkPosture::Append,
        }]);
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Root".to_owned(),
            vec![],
            Some(creation),
            vec![],
        );
        let mut output = BinaryOutput::with_capacity(256);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &11_u16.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        let mut input = BinaryInput::new(&bytes);
        assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
        assert!(input.is_finished());
        bytes[..2].copy_from_slice(&10_u16.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        let mut input = BinaryInput::new(&bytes);
        let old = decode(&mut input, &mut budget).unwrap();
        assert!(old.creation().unwrap().links().is_empty());
        assert!(!input.is_finished());
    }
}
