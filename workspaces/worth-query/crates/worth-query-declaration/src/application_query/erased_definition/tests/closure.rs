//! Portable meaning distinguishes closure and independently refuses non-endorelations.
use super::*;
use crate::application_query::*;
#[test]
fn portable_closure_discriminator_changes_identity_and_requires_endorelation() {
    let source = dynamically_owned_parts();
    let entity = source.root_entity.clone();
    let path = |mode, from: String, to: String| {
        ApplicationQueryRootPathMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRootPathParts {
                start_entity: entity.clone(),
                terminal_entity: entity.clone(),
                guards: vec![],
                steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                    "Next".into(),
                    from,
                    to,
                    RelationDirection::Forward,
                    mode,
                )],
            },
        )
    };
    let mut one = source.clone();
    one.root_paths = vec![path(
        ApplicationQueryRootPathTraversal::OneHop,
        entity.clone(),
        entity.clone(),
    )];
    let mut complete = one.clone();
    complete.root_paths = vec![path(
        ApplicationQueryRootPathTraversal::Closure,
        entity.clone(),
        entity.clone(),
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&complete),
        Ok(())
    );
    assert_ne!(
        ErasedApplicationQueryDefinition::from_untrusted_parts(one).canonical_basis(),
        ErasedApplicationQueryDefinition::from_untrusted_parts(complete.clone()).canonical_basis()
    );
    complete.root_paths = vec![path(
        ApplicationQueryRootPathTraversal::Closure,
        entity.clone(),
        "Other".into(),
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&complete),
        Err(ApplicationQueryDefinitionDenial::InvalidRootPath)
    );
}
