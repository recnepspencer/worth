use super::*;
pub(in crate::application_schema) fn validate_authorization_path(
    path: &ApplicationAuthorizationPath,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_simple_identifier(path.principal_entity())?;
    validate_simple_identifier(path.scope_entity())?;
    for traversal in path.traversals() {
        validate_simple_identifier(traversal.relation())?;
        validate_simple_identifier(traversal.from())?;
        validate_simple_identifier(traversal.to())?;
    }
    for predicate in path.predicates() {
        validate_simple_identifier(predicate.entity())?;
        validate_simple_identifier(predicate.aspect())?;
        validate_field_identifier(predicate.field())?;
    }
    Ok(())
}
