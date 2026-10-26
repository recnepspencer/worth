//! Only existing-anchor direction may invalidate old certified selections.
use super::link_can_change_existing_anchor;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryRootPathMeaning, ApplicationQueryRootPathStep as Step,
        RelationDirection as Path, WorthQueryPortableApplicationQueryRootPathParts,
    },
    application_schema::{
        ApplicationOperationBodyCreation as Creation, ApplicationOperationBodyCreationLink as Link,
        ApplicationOperationBodyCreationLinkDirection as Direction,
        ApplicationOperationBodyCreationLinkPosture as Posture,
        ApplicationOperationBodyCreationPathLink as PathLink,
        ApplicationOperationBodyProgram as Body,
    },
};
#[test]
fn creation_to_principal_changes_reverse_existing_subjects_not_fresh_forward_subjects() {
    use worth_query_declaration::facade::application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyCreationLinkSource as Source,
    };
    let link = Link {
        relation: "CreatedBy".into(),
        from: "Created".into(),
        to: "Principal".into(),
        key: "creator".into(),
        direction: Direction::ToScope,
        posture: Posture::Append,
        source: Source::Principal {
            mutation_binding: "CreateBinding".into(),
            principal_binding: "PrincipalBinding".into(),
            identity: ApplicationFieldBindingLocus::new("Principal", "Identity", "Id"),
        },
    };
    let body = Body::from_untrusted_in_scope(
        "Organization".into(),
        vec![],
        Some(
            Creation::from_untrusted("Created".into(), "new".into(), vec![])
                .with_untrusted_links(vec![link]),
        ),
        vec![],
    );
    let forward = Step::from_untrusted_fields(
        "CreatedBy".into(),
        "Created".into(),
        "Principal".into(),
        Path::Forward,
    );
    let reverse = Step::from_untrusted_fields(
        "CreatedBy".into(),
        "Created".into(),
        "Principal".into(),
        Path::Reverse,
    );
    assert!(!link_can_change_existing_anchor(&body, &forward));
    assert!(link_can_change_existing_anchor(&body, &reverse));
}
#[test]
fn peer_existing_anchor_links_participate_without_touching_fresh_endpoints() {
    for path_link in [false, true] {
        for direction in [Direction::FromScope, Direction::ToScope] {
            let (from, to) = match direction {
                Direction::FromScope => ("Existing", "Created"),
                Direction::ToScope => ("Created", "Existing"),
            };
            let link = Link {
                relation: "Selected".into(),
                from: from.into(),
                to: to.into(),
                key: "selection".into(),
                direction,
                posture: Posture::Append,
                source: Default::default(),
            };
            let peer = Creation::from_untrusted("Created".into(), "peer".into(), vec![]);
            let peer = if path_link {
                peer.with_untrusted_path_links(vec![PathLink {
                    path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
                        WorthQueryPortableApplicationQueryRootPathParts {
                            start_entity: "Scope".into(),
                            terminal_entity: "Existing".into(),
                            steps: vec![Step::from_untrusted_fields(
                                "Anchor".into(),
                                "Scope".into(),
                                "Existing".into(),
                                Path::Forward,
                            )],
                            guards: vec![],
                        },
                    ),
                    link,
                }])
            } else {
                peer.with_untrusted_links(vec![link])
            };
            let body = Body::from_untrusted_in_scope(
                if path_link { "Scope" } else { "Existing" }.into(),
                vec![],
                Some(
                    Creation::from_untrusted("Snapshot".into(), "root".into(), vec![])
                        .with_untrusted_peers(vec![peer]),
                ),
                vec![],
            );
            for path in [Path::Forward, Path::Reverse] {
                let step =
                    Step::from_untrusted_fields("Selected".into(), from.into(), to.into(), path);
                assert_eq!(
                    link_can_change_existing_anchor(&body, &step),
                    matches!(
                        (direction, path),
                        (Direction::FromScope, Path::Forward) | (Direction::ToScope, Path::Reverse)
                    )
                );
            }
            let unrelated =
                Step::from_untrusted_fields("Other".into(), from.into(), to.into(), Path::Forward);
            assert!(!link_can_change_existing_anchor(&body, &unrelated));
        }
    }
}
