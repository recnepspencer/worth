//! Fresh admission checks actual path endpoints, including reverse steps.
use super::*;
use crate::application_query::{
    ApplicationQueryRootPath, ApplicationQueryRootPathMeaning, ApplicationQueryRootPathStep,
    WorthQueryPortableApplicationQueryRootPathParts,
};
use crate::application_schema::{ApplicationRelationIntegrity, ApplicationRelationRef};

#[test]
fn typed_root_path_preserves_forward_and_reverse_endpoint_meaning() {
    let path = typed_path();
    let mut parts = dynamically_owned_parts();
    parts.root_paths = vec![path.clone()];
    assert_eq!(validate_portable_application_query_freshly(&parts), Ok(()));
    assert!(validate_portable_application_schema_freshly(schema_with_path(path)).is_ok());
}

#[test]
fn portable_root_path_metadata_cannot_replace_actual_first_or_last_endpoints() {
    let path = typed_path();
    for index in [0, 1] {
        let mut steps = path.steps().to_vec();
        let step = &steps[index];
        // The first step is forward; the last is reverse. Changing the stored
        // source breaks the logical start and terminal respectively.
        steps[index] = ApplicationQueryRootPathStep::from_untrusted_fields(
            step.relation().to_owned(),
            "Other".to_owned(),
            step.to().to_owned(),
            step.direction(),
            step.traversal(),
        );
        let malformed = ApplicationQueryRootPathMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRootPathParts {
                start_entity: path.start_entity().to_owned(),
                terminal_entity: path.terminal_entity().to_owned(),
                steps,
                guards: Vec::new(),
            },
        );
        let mut parts = dynamically_owned_parts();
        parts.root_paths = vec![malformed.clone()];
        assert_eq!(
            validate_portable_application_query_freshly(&parts),
            Err(ApplicationQueryDefinitionDenial::InvalidRootPath),
            "logical endpoint at step {index} must match the actual relation",
        );
        // Every actual relation and entity exists in this schema. Failure is
        // the inconsistent path shape, rather than a missing dependency.
        assert_eq!(
            validate_portable_application_schema_freshly(schema_with_path(malformed)),
            Err(crate::application_schema::ApplicationSchemaDeclarationDenial::InvalidApplicationQuery),
        );
    }
}

fn typed_path() -> ApplicationQueryRootPathMeaning {
    struct Middle;
    struct Outward;
    struct Returning;
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("OwnedEntity");
    let integrity = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let outward =
        ApplicationRelationRef::<Schema, Outward, Entity, Middle>::from_schema_identifiers(
            "Outward",
            "OwnedEntity",
            "Middle",
            integrity,
        );
    let returning =
        ApplicationRelationRef::<Schema, Returning, Entity, Middle>::from_schema_identifiers(
            "Returning",
            "OwnedEntity",
            "Middle",
            integrity,
        );
    ApplicationQueryRootPath::from(entity)
        .forward(outward)
        .reverse(returning)
        .into_meaning()
}

fn schema_with_path(
    path: ApplicationQueryRootPathMeaning,
) -> WorthQueryPortableApplicationSchemaRecord {
    let mut parts = dynamically_owned_parts();
    parts.root_paths = vec![path.clone()];
    let mut members = ["OwnedEntity", "Middle", "Other"]
        .into_iter()
        .map(|entity| ApplicationSchemaMember::Entity {
            entity: entity.to_owned(),
        })
        .collect::<Vec<_>>();
    members.extend(
        path.steps()
            .iter()
            .map(|step| ApplicationSchemaMember::Relation {
                relation: step.relation().to_owned(),
                from: step.from().to_owned(),
                to: step.to().to_owned(),
                integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
                    .into(),
            }),
    );
    members.push(ApplicationSchemaMember::ApplicationQuery {
        definition: ErasedApplicationQueryDefinition::from_untrusted_parts(parts),
    });
    members.sort();
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "WORTH.tests".to_owned(),
            name: "portable-query-path-endpoints".to_owned(),
            major: 1,
            minor: 0,
            members,
            contributions: Vec::new(),
        },
    )
}
