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
) -> Result<
    crate::application_query::ApplicationQueryDefinition<
        Schema,
        Query,
        QueryParameters,
        (),
        QueryEntity,
    >,
    crate::application_query::ApplicationQueryDefinitionDenial,
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
        .build()
}
