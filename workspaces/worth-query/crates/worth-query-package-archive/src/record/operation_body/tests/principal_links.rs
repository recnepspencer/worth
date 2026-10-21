//! V40 preserves principal anchor meaning at every supported creation site.
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryRootPathMeaning, WorthQueryPortableApplicationQueryRootPathParts,
    },
    application_schema::{
        ApplicationOperationBodyChildCreation as Child,
        ApplicationOperationBodyCreationLink as Link,
        ApplicationOperationBodyCreationLinkDirection as Direction,
        ApplicationOperationBodyCreationLinkPosture as Posture,
        ApplicationOperationBodyCreationLinkSource as Source,
        ApplicationOperationBodyCreationPathLink as PathLink,
    },
};
fn principal_link(entity: &str) -> Link {
    Link {
        relation: "CreatedBy".into(),
        from: entity.into(),
        to: "Principal".into(),
        key: "creator".into(),
        direction: Direction::ToScope,
        posture: Posture::Append,
        source: Source::Principal {
            mutation_binding: "CreateBinding".into(),
            principal_binding: "PrincipalBinding".into(),
            identity: ApplicationFieldBindingLocus::new("Principal", "Identity", "Id"),
        },
    }
}
#[test]
fn principal_anchor_v40_roundtrip_includes_root_peer_path_and_child_links_and_refuses_downgrade() {
    let root =
        ApplicationOperationBodyCreation::from_untrusted("Created".into(), "root".into(), vec![])
            .with_untrusted_links(vec![principal_link("Created")])
            .with_untrusted_children(vec![Child {
                parent_key: "root".into(),
                creation: ApplicationOperationBodyCreation::from_untrusted(
                    "Child".into(),
                    "child".into(),
                    vec![],
                )
                .with_untrusted_links(vec![Link {
                    relation: "Children".into(),
                    from: "Created".into(),
                    to: "Child".into(),
                    key: "child".into(),
                    direction: Direction::FromScope,
                    posture: Posture::Append,
                    source: Source::Scope,
                }]),
            }])
            .with_untrusted_peers(vec![ApplicationOperationBodyCreation::from_untrusted(
                "Peer".into(),
                "peer".into(),
                vec![],
            )
            .with_untrusted_links(vec![principal_link("Peer")])])
            .with_untrusted_path_links(vec![PathLink {
                path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
                    WorthQueryPortableApplicationQueryRootPathParts {
                        start_entity: "Organization".into(),
                        terminal_entity: "Organization".into(),
                        steps: vec![],
                        guards: vec![],
                    },
                ),
                link: Link {
                    relation: "Owned".into(),
                    from: "Organization".into(),
                    to: "Created".into(),
                    key: "owner".into(),
                    direction: Direction::FromScope,
                    posture: Posture::Append,
                    source: Source::Scope,
                },
            }]);
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Organization".into(),
        vec![],
        Some(root),
        vec![],
    );
    assert!(body.has_supported_peer_creations());
    assert!(body.has_supported_child_creations());
    let mut output = BinaryOutput::with_capacity(2048);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &40_u16.to_be_bytes());
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
    let denied = super::super::creation_links::write(
        &mut BinaryOutput::with_capacity(256),
        &[principal_link("Created")],
        39,
    )
    .unwrap_err();
    assert_eq!(denied.kind(), Kind::UnsupportedRecordVariant);
}
