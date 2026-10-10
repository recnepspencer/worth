//! Validate and compile every declared query before binding installation.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

pub(super) type QueryMaps = (
    BTreeMap<ApplicationQueryBindingKey, Arc<WorthQueryCompiledApplicationQuery>>,
    BTreeMap<String, Arc<WorthQueryCompiledApplicationQuery>>,
);

pub(super) fn compile_queries<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<QueryMaps, WorthQueryApplicationQueryInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let mut queries = BTreeMap::new();
    let mut names = BTreeMap::new();
    for definition in schema
        .installed_declaration()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::ApplicationQuery { definition } => Some(definition),
            _ => None,
        })
    {
        let key = ApplicationQueryBindingKey::from_definition(definition);
        let compiled = Arc::new(WorthQueryCompiledApplicationQuery::compile(
            schema, definition,
        )?);
        if queries.insert(key, Arc::clone(&compiled)).is_some()
            || names
                .insert(definition.name().to_owned(), compiled)
                .is_some()
        {
            return Err(denial(DenialKind::QueryMeaningChanged, definition.name()));
        }
    }
    Ok((queries, names))
}
