//! The installed projection fixes record names, field order and cardinalities.
use crate::facade::{
    ApplicationFieldPresence, ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
    WorthQueryInstalledGraphReadContract,
};
use worth_foundational::expression_api::{
    ExpressionSchema, ExpressionSchemaBuilder, ExpressionType, ExpressionTypeName,
};
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;

pub(in crate::application_query) fn derive(
    contract: &WorthQueryInstalledGraphReadContract,
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<ExpressionSchema, ()> {
    // Native scalar family alone cannot authorize erasing a declared unit.
    for projection in contract.projections() {
        if !declaration.members().iter().any(|member| matches!(member,
            ApplicationSchemaMember::Field {entity, aspect, field, unit, ..}
            if entity == projection.entity() && aspect == projection.aspect() && field == projection.field() && unit.is_none())) {
            return Err(());
        }
    }
    let (builder, root) = record(ExpressionSchema::builder(), contract, "root")?;
    Ok(builder.operand("source", root).map_err(|_| ())?.build())
}

fn record(
    mut builder: ExpressionSchemaBuilder,
    contract: &WorthQueryInstalledGraphReadContract,
    path: &str,
) -> Result<(ExpressionSchemaBuilder, ExpressionType), ()> {
    let mut fields = Vec::new();
    for projection in contract
        .projections()
        .iter()
        .filter(|projection| projection.parent_path() == path)
    {
        let ty = ExpressionType::from_scalar_family(projection.scalar_family()).map_err(|_| ())?;
        fields.push((
            projection.output_name(),
            if projection.presence() == ApplicationFieldPresence::Optional {
                ExpressionType::option(ty)
            } else {
                ty
            },
        ));
    }
    for relation in contract
        .relations()
        .iter()
        .filter(|relation| relation.parent_path() == path)
    {
        let (next, child) = record(builder, contract, relation.result_path())?;
        builder = next;
        let ty = match relation.cardinality() {
            ApplicationQueryCardinality::ExactlyOne => child,
            ApplicationQueryCardinality::OptionalOne => ExpressionType::option(child),
            ApplicationQueryCardinality::Many => ExpressionType::list(child),
        };
        fields.push((relation.output_name(), ty));
    }
    // Native paths contain traversal indices; Foundation nominal names use
    // qualified identifiers. Native paths are generated, not caller aliases.
    let name = format!(
        "QueryContext::{}",
        path.replace('/', "::").replace(['[', ']'], "_")
    );
    let ty = ExpressionType::Record(ExpressionTypeName::new(&name, 1).map_err(|_| ())?);
    builder = builder.record(&name, 1, fields).map_err(|_| ())?;
    Ok((builder, ty))
}
