//! Portable mutation shape and validation from the real schema fixture.
use super::*;

#[test]
fn portable_mutation_description_rejects_missing_scope_and_duplicate_binding() {
    use crate::application_schema::{
        validate_portable_application_schema_freshly, ApplicationSchemaDeclarationDenial,
        WorthQueryPortableApplicationSchemaRecord,
    };
    let source = MutationSchema::declaration().unwrap();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(source.erased());
    let mut missing_scope = portable.clone().into_parts();
    for member in &mut missing_scope.members {
        if let ApplicationSchemaMember::ApplicationMutation { description } = member {
            let mut parts = description.clone().into_parts();
            parts.scope.field = "MissingScopeField".to_owned();
            *description =
                crate::application_operation::ApplicationMutationDescription::from_untrusted_parts(
                    parts,
                );
        }
    }
    missing_scope.members.sort();
    assert!(matches!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(missing_scope)
        ),
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationMutationDependency)
    ));
    let mut duplicate = portable.into_parts();
    let member = duplicate
        .members
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::ApplicationMutation { .. }))
        .unwrap()
        .clone();
    duplicate.members.push(member);
    duplicate.members.sort();
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(duplicate)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}
