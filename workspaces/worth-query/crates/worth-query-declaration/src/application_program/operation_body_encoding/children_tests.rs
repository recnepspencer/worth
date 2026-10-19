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
