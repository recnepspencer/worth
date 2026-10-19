//! Carriage proof: topology and repeated-kind site multiplicity survive independently of authority.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyChildCreation as Child, ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
};
fn child(parent: &str, from: &str, to: &str, key: &str, relation: &str) -> Child {
    Child {
        parent_key: parent.into(),
        creation: ApplicationOperationBodyCreation::from_untrusted(to.into(), key.into(), vec![])
            .with_untrusted_links(vec![Link {
                relation: relation.into(),
                from: from.into(),
                to: to.into(),
                key: key.into(),
                direction: Direction::FromScope,
                posture: Posture::Append,
            }]),
    }
}
#[test]
fn child_sites_v36_preserve_preorder_parent_links_and_exact_nested_measure() {
    let root = ApplicationOperationBodyCreation::from_untrusted("Row".into(), "row".into(), vec![])
        .with_untrusted_children(vec![
            child("row", "Row", "Cell", "physical", "RowCells"),
            child("physical", "Cell", "Detail", "detail", "CellDetails"),
            child("row", "Row", "Cell", "electronic", "RowCells"),
        ]);
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Ncr".into(),
        vec![],
        Some(root),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(4096);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &36_u16.to_be_bytes());
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
    let mut old = bytes.clone();
    old[..2].copy_from_slice(&35_u16.to_be_bytes());
    let mut input = BinaryInput::new(&old);
    let description = decode(&mut input, &mut budget(&old)).unwrap();
    assert!(description.creation().unwrap().children().is_empty());
    assert!(!input.is_finished());
    // An unsupported nested carrier is refused, not silently flattened by the codec.
    let mut malformed = body.creation().unwrap().children().to_vec();
    malformed[0].creation = malformed[0]
        .creation
        .clone()
        .with_untrusted_children(vec![child(
            "physical",
            "Cell",
            "Detail",
            "nested",
            "CellDetails",
        )]);
    let malformed = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Ncr".into(),
        vec![],
        Some(
            body.creation()
                .unwrap()
                .clone()
                .with_untrusted_children(malformed),
        ),
        vec![],
    );
    assert_eq!(
        write(&mut BinaryOutput::with_capacity(4096), &malformed)
            .unwrap_err()
            .kind(),
        Kind::InvalidRecordShape
    );
}
#[test]
fn multiple_child_parent_links_require_v39_in_root_and_peer_trees() {
    let mut child = child("row", "Row", "Cell", "cell", "Current");
    let mut links = child.creation.links().to_vec();
    links.push(Link {
        relation: "History".into(),
        key: "history".into(),
        ..links[0].clone()
    });
    child.creation = child.creation.with_untrusted_links(links);
    let site = ApplicationOperationBodyCreation::from_untrusted("Row".into(), "row".into(), vec![])
        .with_untrusted_children(vec![child]);
    for peer in [false, true] {
        let root = if peer {
            ApplicationOperationBodyCreation::from_untrusted("Other".into(), "other".into(), vec![])
                .with_untrusted_peers(vec![site.clone()])
        } else {
            site.clone()
        };
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Tenant".into(),
            vec![],
            Some(root),
            vec![],
        );
        let mut output = BinaryOutput::with_capacity(4096);
        write(&mut output, &body).unwrap();
        let bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &39_u16.to_be_bytes());
        let budget = |bytes: &[u8]| {
            RecordDecodeAttempt::begin(
                Default::default(),
                bytes.len() as u64,
                WorthQueryPackageArchiveLimits::DEFAULT,
            )
            .unwrap()
        };
        let mut input = BinaryInput::new(&bytes);
        let mut attempt = budget(&bytes);
        assert_eq!(decode(&mut input, &mut attempt).unwrap(), body);
        assert!(input.is_finished());
        let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
        write(&mut measure, &body).unwrap();
        assert_eq!(measure.nested_entries(), attempt.finish().nested_entries());
        let mut old = bytes;
        old[..2].copy_from_slice(&38_u16.to_be_bytes());
        assert_eq!(
            decode(&mut BinaryInput::new(&old), &mut budget(&old))
                .unwrap_err()
                .kind(),
            Kind::InvalidRecordShape
        );
    }
}
