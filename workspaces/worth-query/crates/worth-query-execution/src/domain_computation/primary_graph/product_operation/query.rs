use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};

use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryControls,
};

/// Work and request limits for an already selected product. These controls
/// cannot select a second basis or change the operation's retained occurrence.
pub struct WorthQueryProductQueryControls<'request> {
    maximum_results: NonZeroUsize,
    maximum_work: NonZeroUsize,
    request: &'request WorthQueryRequestScope,
}

impl<'request> WorthQueryProductQueryControls<'request> {
    pub fn new(
        maximum_results: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request: &'request WorthQueryRequestScope,
    ) -> Self {
        Self {
            maximum_results,
            maximum_work,
            request,
        }
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn admit_application_query<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        self,
        query: &'runtime WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'runtime,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let (application, product, application_basis) = self.into_parts();
        application.admit_application_query(
            query,
            access,
            parameters,
            WorthQueryApplicationQueryControls::product_one_shot(
                product,
                application_basis,
                controls.maximum_results,
                controls.maximum_work,
                controls.request,
            ),
        )
    }
}
