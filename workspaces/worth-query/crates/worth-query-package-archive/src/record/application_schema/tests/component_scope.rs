//! Member40 preserves descriptive path unions exactly; decoding grants no read authority.
use super::{decode_member, encode_member};
use worth_query_declaration::facade::{application_query::*, application_schema::*};
#[test]
fn operation_component_read_tag_round_trips_raw_path_duplicates() {
    let path = ApplicationQueryRootPathMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryRootPathParts {
            start_entity: "Root".into(),
            terminal_entity: "Node".into(),
            steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                "Owns".into(),
                "Root".into(),
                "Node".into(),
                RelationDirection::Forward,
                ApplicationQueryRootPathTraversal::OneHop,
            )],
            guards: vec![],
        },
    );
    // A duplicate is deliberately retained for fresh schema admission to reject.
    let original = ApplicationSchemaMember::OperationComponentRead {
        definition: ApplicationOperationComponentReadMeaning::from_untrusted_parts(
            WorthQueryPortableOperationComponentReadParts {
                operation: "Edit".into(),
                set: "Shells".into(),
                root: "Root".into(),
                member: "Node".into(),
                paths: vec![path.clone(), path],
            },
        ),
    };
    let bytes = encode_member(&original);
    assert_eq!(&bytes[..2], &40_u16.to_be_bytes());
    assert_eq!(decode_member(&bytes), original);
}
