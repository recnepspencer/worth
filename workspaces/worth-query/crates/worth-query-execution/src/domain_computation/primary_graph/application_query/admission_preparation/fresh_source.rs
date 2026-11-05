//! Fresh source parameters on the original invalidation admission.
use super::*;
use worth_query_admission::facade::application_query::admit_application_query_parameters;

pub(in crate::domain_computation::primary_graph::application_query) trait FreshSourceScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn prepare_source_query_admission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        (
            WorthQueryAdmittedApplicationQueryParameters,
            WorthQueryApplicationQueryControls<'a, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        self.validate_application_query_admission(query, access, &controls)?;
        let parameters = admit_application_query_parameters(query, parameters).map_err(|stop| {
            WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::Parameter(stop.kind()),
                stop.parameter(),
            )
        })?;
        Ok((parameters, controls))
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > FreshSourceScopedOps<Schema> for HostRuntime
{
}
