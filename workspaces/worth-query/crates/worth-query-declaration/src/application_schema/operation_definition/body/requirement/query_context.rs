//! A query context is one exact source expectation, not caller-supplied records.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementOperands,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationSourceExpectation},
    application_query::{
        ApplicationQueryCardinality, ApplicationQueryDefinition, ErasedApplicationQueryDefinition,
    },
    application_schema::{ApplicationSchema, ApplicationSchemaMember as Member},
};

impl<Schema: ApplicationSchema, Operation>
    ApplicationOperationBodyRequirementOperands<Schema, Operation>
{
    /// Consumes a parameter-free exactly-one query declaration. Its native
    /// observed source must accompany fresh guidance and mutation execution.
    pub fn query_context<Binding, Query, Parameters, QueryResult, Scope>(
        self,
        name: &str,
        query: ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation>,
        Binding::SourceExpectation: ApplicationMutationSourceExpectation<Schema, Query = Query>,
    {
        if !query.parameters().is_empty()
            || query.cardinality() != ApplicationQueryCardinality::ExactlyOne
        {
            return Err(Denial::UnsupportedQueryContext);
        }
        self.push(
            name,
            Source::QueryContext {
                mutation_binding: Binding::IDENTITY.to_owned(),
                query: Box::new(query.into_erased()),
            },
        )
    }
}

pub(in crate::application_schema::operation_definition::body) fn is_declared(
    operation: &str,
    binding: &str,
    query: &ErasedApplicationQueryDefinition,
    members: &[Member],
) -> bool {
    query.parameters().is_empty() && query.cardinality() == ApplicationQueryCardinality::ExactlyOne
        && members.iter().any(|member| matches!(member, Member::ApplicationQuery { definition } if definition == query))
        && members.iter().any(|member| matches!(member, Member::ApplicationMutation { description } if description.operation() == operation && description.binding_identity().as_str() == binding && description.scope().entity == query.scope_entity()))
        && query.result_shape().expression_context_schema(members).is_ok()
}
