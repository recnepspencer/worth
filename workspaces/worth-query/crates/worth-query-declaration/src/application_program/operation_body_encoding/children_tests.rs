//! Canonical meaning, not execution authority: lawful parent changes retain the same effect footprint.
use super::operation_body_record;
use crate::application_schema::{
    ApplicationOperationBodyChildCreation as Child, ApplicationOperationBodyCreation as Creation,
    ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
    ApplicationOperationBodyProgram as Body,
};
fn child(parent: &str, from: &str, to: &str, key: &str, relation: &str) -> Child {
    Child {
        parent_key: parent.into(),
        creation: Creation::from_untrusted(to.into(), key.into(), vec![]).with_untrusted_links(
            vec![Link {
                relation: relation.into(),
                from: from.into(),
                to: to.into(),
                key: key.into(),
                direction: Direction::FromScope,
                posture: Posture::Append,
            }],
        ),
    }
}
fn body(parent: &str) -> Body {
    Body::from_untrusted_in_scope(
        "Tenant".into(),
        vec![],
        Some(
            Creation::from_untrusted("Row".into(), "row".into(), vec![]).with_untrusted_children(
                vec![
                    child("row", "Row", "Cell", "left", "RowCells"),
                    child("row", "Row", "Cell", "right", "RowCells"),
                    child(parent, "Cell", "Detail", "detail", "CellDetails"),
                ],
            ),
        ),
        vec![],
    )
}
#[test]
fn child_sites_change_canonical_meaning_when_parent_changes_without_changing_field_footprint() {
    let left = body("left");
    let right = body("right");
    assert!(left.has_supported_child_creations());
    assert!(right.has_supported_child_creations());
    assert_ne!(operation_body_record(&left), operation_body_record(&right));
    let targets = |body: &Body| {
        body.creation()
            .unwrap()
            .initialization_sites()
            .flat_map(|site| site.links().iter().map(|link| link.program_target()))
            .collect::<Vec<_>>()
    };
    assert_eq!(targets(&left), targets(&right));
}
#[test]
fn every_child_parent_link_survives_root_and_peer_program_meaning() {
    let multiple = |relation: &str| {
        let original = body("left");
        let mut children = original.creation().unwrap().children().to_vec();
        let mut links = children[0].creation.links().to_vec();
        links.push(Link {
            relation: relation.into(),
            key: "archive".into(),
            ..links[0].clone()
        });
        children[0].creation = children[0].creation.clone().with_untrusted_links(links);
        original
            .creation()
            .unwrap()
            .clone()
            .with_untrusted_children(children)
    };
    for peer in [false, true] {
        let make = |relation| {
            let site = multiple(relation);
            let root = if peer {
                Creation::from_untrusted("Other".into(), "other".into(), vec![])
                    .with_untrusted_peers(vec![site])
            } else {
                site
            };
            Body::from_untrusted_in_scope("Tenant".into(), vec![], Some(root), vec![])
        };
        let left = make("ArchiveA");
        let right = make("ArchiveB");
        assert!(left.has_multiple_child_links());
        assert!(left.has_supported_child_creations());
        assert_ne!(operation_body_record(&left), operation_body_record(&right));
    }
}
