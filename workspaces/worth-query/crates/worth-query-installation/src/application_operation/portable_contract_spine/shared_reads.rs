//! Portable scope contents compile once; operation records carry references.
use super::*;
pub(crate) fn compile_shared_read_scopes(
    schema: &str,
    native: &[WorthQueryPortableNativeAspectContractRecord],
    members: &[ApplicationSchemaMember],
    definition: &worth_query_declaration::facade::application_schema::ApplicationDecisionReadScopeMeaning,
) -> SpineResult<Vec<WorthQueryPortableOperationGraphReadScope>> {
    let mut reads = worth_query_declaration::facade::application_schema::application_decision_read_scope_physical_targets(members, definition);
    let derived = derived_reads::append_relations(
        members,
        definition.derived_relations().iter().map(String::as_str),
        &mut reads,
    )?;
    let mut scopes = compile_reads(schema, native, reads)?;
    scopes.extend(derived.into_iter().map(|relation| {
        WorthQueryPortableOperationGraphReadScope::DerivedSource {
            schema: schema.to_owned(),
            relation,
        }
    }));
    Ok(scopes)
}
