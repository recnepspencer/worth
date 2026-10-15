//! V11 carries creation linkage without reinterpreting older body bytes.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLink,
    ApplicationOperationBodyCreationLinkDirection as Direction,
};
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
