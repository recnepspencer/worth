//! Sealed test doors over this module's private members.
use super::*;

pub trait AdmissionPreparationDoorsOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    fn validate_installed_query_for_test<Query, Parameters, QueryResult, Scope>(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        Self::validate_installed_query::<Query, Parameters, QueryResult, Scope>(self, query)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > AdmissionPreparationDoorsOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}
