//! V41 carries a created-to-created link; older profiles cannot express it.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
    ApplicationOperationBodyCreationLinkSource as Source,
};
fn link(relation: &str, from: &str, direction: Direction) -> Link {
    Link {
        relation: relation.into(),
        from: from.into(),
        to: "Project".into(),
        key: relation.to_lowercase(),
        direction,
        posture: Posture::Append,
        source: Source::Scope,
    }
}
#[test]
fn self_edge_v41_roundtrips_beside_a_scope_link_and_refuses_older_profiles() {
    let self_edge = link("Watches", "Project", Direction::SelfEdge);
    let creation = ApplicationOperationBodyCreation::from_untrusted(
        "Project".into(),
        "project".into(),
        vec![],
    )
    .with_untrusted_links(vec![
        link("Contains", "Workspace", Direction::FromScope),
        self_edge.clone(),
    ]);
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Workspace".into(),
        vec![],
        Some(creation),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &41_u16.to_be_bytes());
    let budget = |bytes: &[u8]| {
        RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap()
    };
    let mut decoded = budget(&bytes);
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut decoded).unwrap(), body);
    assert!(input.is_finished());
    let mut measured = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measured, &body).unwrap();
    assert_eq!(measured.nested_entries(), decoded.finish().nested_entries());
    let denied = super::super::creation_links::write(
        &mut BinaryOutput::with_capacity(256),
        &[self_edge],
        40,
    )
    .unwrap_err();
    assert_eq!(denied.kind(), Kind::UnsupportedRecordVariant);
    // Direction tag 3 read under a v40 header is not reinterpreted.
    bytes[..2].copy_from_slice(&40_u16.to_be_bytes());
    let refused = decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes)).unwrap_err();
    assert_eq!(refused.kind(), Kind::UnsupportedRecordVariant);
}
