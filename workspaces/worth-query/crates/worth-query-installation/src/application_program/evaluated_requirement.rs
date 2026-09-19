use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_program::{ApplicationEvaluatedRequirement, ApplicationEvaluatedRequirementRule},
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
};

/// Installed access to one exact rule evaluator. This is descriptive and grants
/// no operation execution or authorization authority.
pub struct WorthQueryInstalledEvaluatedRequirement<Schema, Operation, Rule> {
    schema_binding: ApplicationSchemaBindingIdentity,
    marker: PhantomData<fn() -> (Schema, Operation, Rule)>,
}

impl<Schema, Operation, Rule> WorthQueryInstalledEvaluatedRequirement<Schema, Operation, Rule>
where
    Schema: ApplicationSchema,
    Operation:
        worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
    Rule: ApplicationEvaluatedRequirementRule<Schema, Operation>,
{
    pub(super) const fn new(schema_binding: ApplicationSchemaBindingIdentity) -> Self {
        Self {
            schema_binding,
            marker: PhantomData,
        }
    }

    pub const fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn evaluate(
        &self,
        context: &Rule::Context,
    ) -> ApplicationEvaluatedRequirement<Rule::Requirement, Rule::Finding> {
        Rule::evaluate(context)
    }
}
