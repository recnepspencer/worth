use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};

use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryControls,
};

mod batch;
mod retained;
pub use batch::WorthQueryRetainedBatchQueryAdmissionDenial;

/// Work and request limits for an already selected product. These controls
/// cannot select a second basis or change the operation's retained occurrence.
pub struct WorthQueryProductQueryControls<'request> {
    maximum_results: NonZeroUsize,
    maximum_work: Option<NonZeroUsize>,
    request: &'request WorthQueryRequestScope,
}

impl<'request> WorthQueryProductQueryControls<'request> {
    pub fn new(
        maximum_results: NonZeroUsize,
        maximum_work: Option<NonZeroUsize>,
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
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), query.name())),
        };
        let execution = &request;

        let (application, product, application_basis) = self.into_parts();
        application.admit_application_query(
            execution,
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

    /// Admits a one-shot query whose authorization and truth bases differ.
    /// The receiver is the freshly selected security product; `retained` is
    /// the exact owner-retained product supplying query truth.
    pub fn admit_retained_application_query<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
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
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), query.name())),
        };
        let execution = &request;

        let (application, controls) =
            self.retained_query_controls(execution, retained, query.name(), controls)?;
        application.admit_application_query(execution, query, access, parameters, controls)
    }

    pub fn admit_application_query_continuation<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), query.name())),
        };
        let execution = &request;

        let (application, product, application_basis) = self.into_parts();
        application.admit_application_query(
            execution,
            query,
            access,
            parameters,
            WorthQueryApplicationQueryControls::product_continuation(
                product.into_read_lease(),
                application_basis,
                controls.maximum_results,
                controls.maximum_work,
                controls.request,
            ),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit_governed_application_query<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Capability,
        Operation,
        Input,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
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
    >
    where
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
    {
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), query.name())),
        };
        let execution = &request;

        let (application, product, application_basis) = self.into_parts();
        application.admit_governed_application_query(
            execution,
            query,
            access,
            capability,
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

    #[allow(clippy::too_many_arguments)]
    pub fn admit_governed_application_query_continuation<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Capability,
        Operation,
        Input,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
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
    >
    where
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
    {
        let request = match phase.execution_request_for(&self.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)), query.name())),
        };
        let execution = &request;

        let (application, product, application_basis) = self.into_parts();
        application.admit_governed_application_query(
            execution,
            query,
            access,
            capability,
            parameters,
            WorthQueryApplicationQueryControls::product_continuation(
                product.into_read_lease(),
                application_basis,
                controls.maximum_results,
                controls.maximum_work,
                controls.request,
            ),
        )
    }
}

/// Preserve the original request stop instead of relabeling it as a basis mismatch.
fn selection_denial(
    stop: crate::basis::WorthQueryProductBranchAdmissionDenial,
    query_name: &str,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use crate::basis::WorthQueryProductBranchAdmissionDenial as Branch;
    use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind as Kind;
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
    let kind = match stop {
        Branch::Stopped(Err(Stop::Cancelled)) => Kind::Cancelled,
        Branch::Stopped(Err(Stop::TimedOut)) => Kind::DeadlineExceeded,
        _ => Kind::ForeignBasis,
    };
    WorthQueryApplicationQueryAdmissionDenial::new(kind, query_name)
}
