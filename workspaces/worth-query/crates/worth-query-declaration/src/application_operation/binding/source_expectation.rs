use std::marker::PhantomData;

use crate::{
    application_query::ApplicationQueryMarkerIdentity, application_schema::ApplicationSchema,
};

pub trait ApplicationMutationSourceExpectation<Schema>: 'static
where
    Schema: ApplicationSchema,
{
    type Query: 'static;

    const QUERY_IDENTIFIER: Option<&'static str>;
}

pub struct NoApplicationMutationSource;

pub struct ApplicationQueryMutationSource<Query>(PhantomData<fn() -> Query>);

impl<Schema> ApplicationMutationSourceExpectation<Schema> for NoApplicationMutationSource
where
    Schema: ApplicationSchema,
{
    type Query = NoApplicationMutationSource;

    const QUERY_IDENTIFIER: Option<&'static str> = None;
}

impl<Schema, Query> ApplicationMutationSourceExpectation<Schema>
    for ApplicationQueryMutationSource<Query>
where
    Schema: ApplicationSchema,
    Query: ApplicationQueryMarkerIdentity<Schema> + 'static,
{
    type Query = Query;

    const QUERY_IDENTIFIER: Option<&'static str> = Some(Query::IDENTIFIER);
}
