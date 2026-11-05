use worth_query_declaration::facade::application_query::ApplicationQueryParameterRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::Activity;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::ActivityFacts;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::ActivitySequenceResultSlot;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::CrossRootQuery;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::StatusParameter;

pub fn status_parameter<Query>(
) -> ApplicationQueryParameterRef<Query, StatusParameter, StringApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("status")
}

pub fn activity_sequence_result_field() -> ApplicationQueryResultFieldRef<
    CrossRootQuery,
    ActivitySequenceResultSlot,
    IdentityExecutionSchema,
    Activity,
    ActivityFacts,
    ActivitySequence,
    u64,
    worth_query_declaration::facade::application_schema::ReadOnly,
    worth_query_declaration::facade::application_schema::NoEqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::reference())
}
