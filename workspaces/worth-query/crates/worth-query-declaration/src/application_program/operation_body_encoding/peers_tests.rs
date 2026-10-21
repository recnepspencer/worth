//! Program identity preserves initializer meaning even when effect footprints match.
use super::operation_body_record;
use crate::{
    application_query::{
        ApplicationQueryRootPathDirection as Path, ApplicationQueryRootPathMeaning,
        ApplicationQueryRootPathStep as Step, WorthQueryPortableApplicationQueryRootPathParts,
    },
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField as Field,
        ApplicationOperationBodyCreation as Creation, ApplicationOperationBodyCreationLink as Link,
        ApplicationOperationBodyCreationLinkDirection as Direction,
        ApplicationOperationBodyCreationLinkPosture as Posture,
        ApplicationOperationBodyCreationPathLink as PathLink,
        ApplicationOperationBodyProgram as Body,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
fn body(member: &str, relation: &str) -> Body {
    let peer = Creation::from_untrusted(
        "Journal".into(),
        "audit".into(),
        vec![Field::Input {
            field: ApplicationFieldBindingLocus::new("Journal", "Facts", "Name"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into()),
            scalar_family: worth_foundational::facade::ScalarAspectType::String,
            member: member.into(),
        }],
    )
    .with_untrusted_path_links(vec![PathLink {
        path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRootPathParts {
                start_entity: "Row".into(),
                terminal_entity: "Parent".into(),
                steps: vec![Step::from_untrusted_fields(
                    relation.into(),
                    "Parent".into(),
                    "Row".into(),
                    Path::Reverse,
                )],
                guards: vec![],
            },
        ),
        link: Link {
            relation: "Journal".into(),
            from: "Parent".into(),
            to: "Journal".into(),
            key: "entry".into(),
            direction: Direction::FromScope,
            posture: Posture::Append,
            source: Default::default(),
        },
    }]);
    Body::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        Some(
            Creation::from_untrusted("Snapshot".into(), "current".into(), vec![])
                .with_untrusted_peers(vec![peer]),
        ),
        vec![],
    )
}
#[test]
fn peer_inputs_and_existing_anchor_paths_are_canonical_program_meaning() {
    let original = body("name", "Rows");
    for changed in [body("label", "Rows"), body("name", "OtherRows")] {
        assert_eq!(
            original.creation().unwrap().peers()[0].path_links()[0]
                .link
                .program_target(),
            changed.creation().unwrap().peers()[0].path_links()[0]
                .link
                .program_target()
        );
        assert_ne!(
            operation_body_record(&original),
            operation_body_record(&changed)
        );
    }
}
