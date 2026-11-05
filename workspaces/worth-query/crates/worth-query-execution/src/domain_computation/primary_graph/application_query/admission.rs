use crate::domain_computation::primary_graph::application_query::admission_preparation::AdmissionPreparationParentOps;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryParameterSet, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use {
    super::WorthQueryAdmittedApplicationQueryPlan, super::WorthQueryApplicationQueryAccessContext,
    super::WorthQueryApplicationQueryControls,
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial,
};

mod access_observation;
mod authorization;
mod correction_disclosure;
mod denial;
mod finish;
mod funded_parameters;
pub(in crate::domain_computation::primary_graph) use funded_parameters::capture_parameters;

mod governed_access;
mod live_readmission;
pub(crate) mod permission;
mod source_readmission;
mod work_limit;
pub(in crate::domain_computation::primary_graph::application_query) use governed_access::governance_denial;
pub(in crate::domain_computation::primary_graph::application_query) use governed_access::prepare_governed_access;
pub(in crate::domain_computation::primary_graph) use permission::SelectedIssuedAccessRoot;
pub(in crate::domain_computation::primary_graph) use permission::{
    FreshQueryPermissionStop, PreparedApplicationQueryPermission,
};

pub(in crate::domain_computation::primary_graph) trait AdmissionScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn admit_application_query<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
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
        let (parameters, controls) =
            self.prepare_application_query_admission(query, access, parameters, controls)?;
        self.finish_application_query_admission(
            execution, query, access, parameters, controls, None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn admit_governed_application_query<
        'a,
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
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
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
        let pending = prepare_governed_access(
            self.application_query_host(),
            query,
            access,
            capability,
            controls.request_scope(),
        )?;
        let (parameters, controls) =
            self.prepare_application_query_admission(query, access, parameters, controls)?;
        self.finish_application_query_admission(
            execution,
            query,
            access,
            parameters,
            controls,
            Some(pending),
        )
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > AdmissionScopedOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

pub(crate) mod court_doors_for_test;
pub(in crate::domain_computation::primary_graph::application_query) use access_observation::AccessObservationScopedOps;
pub(in crate::domain_computation::primary_graph) use correction_disclosure::CorrectionDisclosureScopedOps;
pub(in crate::domain_computation::primary_graph) use finish::OrdinaryScopedOps;
pub(in crate::domain_computation::primary_graph) use finish::PreparedPermissionScopedOps;
pub(in crate::domain_computation::primary_graph) use finish::ReadmittedScopedOps;
pub(in crate::domain_computation::primary_graph::application_query) use live_readmission::LiveReadmissionScopedOps;
pub(in crate::domain_computation::primary_graph) use permission::AccessScopedOps;
pub(in crate::domain_computation::primary_graph) use permission::FreshScopedOps;
pub(in crate::domain_computation::primary_graph) use permission::ReadAuthorizationScopedOps;
pub(in crate::domain_computation::primary_graph::application_query) use permission::SourceParametersScopedOps;
pub(in crate::domain_computation::primary_graph) use source_readmission::SourceReadmissionScopedOps;
pub use work_limit::WorkLimitOps;
