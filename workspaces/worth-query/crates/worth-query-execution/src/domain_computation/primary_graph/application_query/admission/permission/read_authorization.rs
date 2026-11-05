//! Observe READ on the genuine issued source root, independently of payload policy.
use super::*;
use crate::domain_computation::authorization::RequirementObservationScopedOps;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind;

type ReadAuthorization = (
    WorthQueryRetainedAuthorizationDecisionFacts,
    WorthQueryApplicationAuthorizationWorkEvidence,
);

pub(in crate::domain_computation::primary_graph) trait ReadAuthorizationScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn observe_issued_query_read_authorization<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        root: &SelectedIssuedAccessRoot<'_, Schema, Principal, PrincipalIdentity, Scope>,
        session: &WorthQueryPreparedProductReadSessionIdentity,
    ) -> Result<ReadAuthorization, WorthQueryApplicationQueryAdmissionDenial> {
        self.observe_issued_query_read_authorization_with_native(
            execution,
            query,
            root,
            session,
            &mut ApplicationQueryNativeAccess::acquire(),
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > ReadAuthorizationScopedOps<Schema> for HostRuntime
{
}

pub(super) trait ReadAuthorizationParentOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn observe_issued_query_read_authorization_with_native<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        root: &SelectedIssuedAccessRoot<'_, Schema, Principal, PrincipalIdentity, Scope>,
        session: &WorthQueryPreparedProductReadSessionIdentity,
        native: &mut ApplicationQueryNativeAccess<'_>,
    ) -> Result<ReadAuthorization, WorthQueryApplicationQueryAdmissionDenial> {
        let graph = self.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                query.name(),
            )
        })?;
        let access = WorthQueryApplicationQueryAccessContext::new(root.principal, root.scope);
        let principal_currentness = principal_dependency::capture(
            self.application_query_host(),
            access.principal(),
            session.identity(),
        )?;
        let (authorization, work) = match query.authorization() {
            WorthQueryInstalledApplicationQueryAuthorization::Public => (
                WorthQueryRetainedAuthorizationDecisionFacts::principal(principal_currentness),
                WorthQueryApplicationAuthorizationWorkEvidence::from_dependencies(&[]),
            ),
            WorthQueryInstalledApplicationQueryAuthorization::Ability(requirement) => {
                // Preserve the original Native observer and its original denial.
                let decisions = native.with_runtime_mut(
                    graph,
                    execution,
                    || crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                        crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(
                            crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind::ForeignRuntime,
                        ),
                        query.name(),
                    ),
                    |relational| self.observe_query_authorization_requirement(
                        execution, session.identity(), relational, root.snapshot.clone(), query,
                        &access, requirement,
                    ),
                ).map_err(WorthQueryApplicationQueryAdmissionDenial::from_authorization)?;
                let work =
                    WorthQueryApplicationAuthorizationWorkEvidence::from_dependencies(&decisions);
                (
                    WorthQueryRetainedAuthorizationDecisionFacts::abilities(
                        principal_currentness,
                        decisions,
                    ),
                    work,
                )
            }
        };
        if !authorization.belongs_to_session(session.identity()) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable,
                query.name(),
            ));
        }
        Ok((authorization, work))
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > ReadAuthorizationParentOps<Schema> for HostRuntime
{
}
