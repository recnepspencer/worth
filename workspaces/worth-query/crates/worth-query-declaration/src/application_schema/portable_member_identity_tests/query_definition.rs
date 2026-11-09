use super::{QueryEntity, QueryParameters, Schema};
use crate::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultShapeBuilder,
};
use crate::application_schema::ApplicationEntityRef;

pub(super) fn query_definition<Query>(
    reference: crate::application_query::ApplicationQueryReference<
        Schema,
        Query,
        QueryParameters,
        (),
        QueryEntity,
    >,
    entity: ApplicationEntityRef<Schema, QueryEntity>,
) -> crate::application_query::ApplicationQueryDefinitionBuilder<
    Schema,
    Query,
    QueryParameters,
    (),
    QueryEntity,
>
where
    Query: crate::application_query::ApplicationQueryMarkerIdentity<Schema>,
    Query::ResultBinding: crate::application_schema::ApplicationStructuredValueBinding<Value = ()>,
{
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(entity)
        .scope(entity)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                Query,
                QueryEntity,
                (),
                Query::ResultBinding,
            >::new(entity)
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
