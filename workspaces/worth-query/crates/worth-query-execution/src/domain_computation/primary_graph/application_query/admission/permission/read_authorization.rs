//! Observe READ on the genuine issued source root, independently of payload policy.
use super::*;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;

type ReadAuthorization = (
    WorthQueryRetainedAuthorizationDecisionFacts,
    WorthQueryApplicationAuthorizationWorkEvidence,
);

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// This observes READ only. The caller still admits its metadata contract
    /// and consumes both decisions in its own actual session.
    pub(in crate::domain_computation::primary_graph) fn observe_issued_query_read_authorization<
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

    pub(super) fn observe_issued_query_read_authorization_with_native<
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
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                query.name(),
            )
        })?;
        let access = WorthQueryApplicationQueryAccessContext::new(root.principal, root.scope);
        let principal_currentness =
            principal_dependency::capture(self, access.principal(), session.identity())?;
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
                        crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(
                            crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::ForeignRuntime,
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
