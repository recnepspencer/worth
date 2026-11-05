//! Sealed test doors over this module's private members.
use super::*;

pub trait CorrectionDisclosureDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    #[doc(hidden)]
    fn execute_correction_disclosure_for_test<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        original_subject: EntityId,
    ) -> Result<bool, WorthQueryApplicationOneShotDenial> {
        Self::execute_correction_disclosure::<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(self, execution, plan, original_subject)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > CorrectionDisclosureDoorsOps<Schema> for HostRuntime
{
}
