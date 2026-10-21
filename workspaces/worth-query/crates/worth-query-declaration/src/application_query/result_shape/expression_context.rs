//! Pure record vocabulary shared by authored requirements and installed reads.
use super::ApplicationQueryResultShape;
use crate::application_query::{ApplicationQueryCardinality, ApplicationQueryResultField};
use crate::application_schema::{ApplicationFieldPresence, ApplicationSchemaMember};
use worth_foundational::expression_api::{
    ExpressionDenial, ExpressionSchema, ExpressionSchemaBuilder, ExpressionType, ExpressionTypeName,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationQueryExpressionContextDenial {
    UndeclaredOrUnitBearingField,
    Expression(ExpressionDenial),
}
impl From<ExpressionDenial> for ApplicationQueryExpressionContextDenial {
    fn from(cause: ExpressionDenial) -> Self {
        Self::Expression(cause)
    }
}

impl ApplicationQueryResultShape {
    /// Derives the unitless native expression profile against a declared schema.
    /// This validates descriptive meaning; it grants no query or mutation access.
    pub fn expression_context_schema(
        &self,
        members: &[ApplicationSchemaMember],
    ) -> Result<ExpressionSchema, ApplicationQueryExpressionContextDenial> {
        let validate = |projection: &ApplicationQueryResultField| {
            if members.iter().any(|member| matches!(member,
                ApplicationSchemaMember::Field {entity, aspect, field, unit: None, ..}
                if entity == projection.entity() && aspect == projection.aspect() && field == projection.field())) { Ok(()) }
            else { Err(ApplicationQueryExpressionContextDenial::UndeclaredOrUnitBearingField) }
        };
        let (builder, root) = record(ExpressionSchema::builder(), self, "root", &validate)?;
        Ok(builder.operand("source", root)?.build())
    }

    // Authoring admits only pure vocabulary here. Schema closure subsequently
    // applies the checked profile above before any installation can execute it.
    pub(crate) fn append_expression_context(
        &self,
        builder: ExpressionSchemaBuilder,
    ) -> Result<(ExpressionSchemaBuilder, ExpressionType), ApplicationQueryExpressionContextDenial>
    {
        record(builder, self, "root", &|_| Ok(()))
    }
}

fn record(
    mut builder: ExpressionSchemaBuilder,
    shape: &ApplicationQueryResultShape,
    path: &str,
    validate: &impl Fn(
        &ApplicationQueryResultField,
    ) -> Result<(), ApplicationQueryExpressionContextDenial>,
) -> Result<(ExpressionSchemaBuilder, ExpressionType), ApplicationQueryExpressionContextDenial> {
    let mut fields = Vec::new();
    for field in shape.fields() {
        validate(field)?;
        let ty = ExpressionType::from_scalar_family(field.scalar_family())?;
        fields.push((
            field.output_name(),
            if field.presence() == ApplicationFieldPresence::Optional {
                ExpressionType::option(ty)
            } else {
                ty
            },
        ));
    }
    for (index, relation) in shape.relations().iter().enumerate() {
        let (next, child) = record(
            builder,
            relation.nested_shape(),
            &format!("{path}::relation_{index}_"),
            validate,
        )?;
        builder = next;
        fields.push((
            relation.output_name(),
            match relation.cardinality() {
                ApplicationQueryCardinality::ExactlyOne => child,
                ApplicationQueryCardinality::OptionalOne => ExpressionType::option(child),
                ApplicationQueryCardinality::Many => ExpressionType::list(child),
            },
        ));
    }
    let name = format!("QueryContext::{path}");
    let ty = ExpressionType::Record(ExpressionTypeName::new(&name, 1)?);
    Ok((builder.record(&name, 1, fields)?, ty))
}
