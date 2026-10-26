//! New site/path transport has exact nested budgeting without reinterpreting old bodies.
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryRootPathMeaning, ApplicationQueryRootPathStep, RelationDirection,
        WorthQueryPortableApplicationQueryRootPathParts,
    },
    application_schema::{
        ApplicationOperationBodyCreationLink as Link,
        ApplicationOperationBodyCreationLinkDirection as Direction,
        ApplicationOperationBodyCreationLinkPosture as Posture,
        ApplicationOperationBodyCreationPathLink as PathLink,
    },
};
#[test]
fn v38_preserves_independent_root_and_existing_anchor_without_changing_old_profile() {
    let path = PathLink {
        path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRootPathParts {
                start_entity: "Row".into(),
                terminal_entity: "Parent".into(),
                steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                    "Rows".into(),
                    "Parent".into(),
                    "Row".into(),
                    RelationDirection::Reverse,
                )],
                guards: vec![],
            },
        ),
        link: Link {
            relation: "Audit".into(),
            from: "Parent".into(),
            to: "Journal".into(),
            key: "audit".into(),
            direction: Direction::FromScope,
            posture: Posture::Append,
            source: Default::default(),
        },
    };
    let root = ApplicationOperationBodyCreation::from_untrusted(
        "Snapshot".into(),
        "current".into(),
        vec![],
    );
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        Some(root.clone()),
        vec![],
    );
    let mut old = BinaryOutput::with_capacity(4096);
    write(&mut old, &body).unwrap();
    let old = old.into_bytes();
    assert_eq!(&old[..2], &8_u16.to_be_bytes());
    let peer =
        ApplicationOperationBodyCreation::from_untrusted("Journal".into(), "audit".into(), vec![])
            .with_untrusted_path_links(vec![path]);
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        Some(root.with_untrusted_peers(vec![peer.clone()])),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(4096);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &38_u16.to_be_bytes());
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
    let malformed = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        Some(
            body.creation()
                .unwrap()
                .clone()
                .with_untrusted_peers(vec![peer.with_untrusted_reuse_current_if_unchanged(true)]),
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
