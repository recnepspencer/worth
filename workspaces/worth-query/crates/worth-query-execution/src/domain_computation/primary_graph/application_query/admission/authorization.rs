use super::{denial::graph_work_denial, governed_access::governance_denial};
use crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts;
use crate::domain_computation::primary_graph::{
    application_query::{
        disclosure::{
            admit_application_query_governance, WorthQueryAdmittedApplicationDisclosureContract,
            WorthQueryApplicationGovernanceBinding, WorthQueryApplicationQueryGovernance,
            WorthQueryPendingApplicationQueryGovernance,
        },
        WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryAdmissionDenial,
    },
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
pub(super) struct AdmittedApplicationQueryAuthorities {
    pub(super) authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    pub(super) authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    pub(super) governance: WorthQueryApplicationQueryGovernance,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn admit_application_query_authorities<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        graph_work: &mut WorthQueryManagedGraphWorkSession,
        basis: &super::super::basis::WorthQueryApplicationQueryBasisCustody,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: &WorthQueryAdmittedApplicationQueryParameters,
        mut pending_governance: Option<WorthQueryPendingApplicationQueryGovernance>,
        disclosure: WorthQueryAdmittedApplicationDisclosureContract,
    ) -> Result<AdmittedApplicationQueryAuthorities, WorthQueryApplicationQueryAdmissionDenial>
    {
        if let Some(pending) = pending_governance.as_mut() {
            self.refresh_capability_authorization_for_graph_work(
                pending.authorization_mut(),
                graph_work,
            )
            .map_err(WorthQueryApplicationQueryAdmissionDenial::from_authorization)?;
        }
        let governance = admit_application_query_governance(
            disclosure,
            pending_governance,
            WorthQueryApplicationGovernanceBinding::from_session(
                graph_work,
                query.identity().clone(),
                *parameters.identity(),
                access.principal().principal_entity_id(),
                access.scope().entity_id(),
            ),
        )
        .map_err(|kind| governance_denial(kind, query.name()))?;
        let security = self.admit_product_security_basis(basis.product())
            .map_err(|denial| WorthQueryApplicationQueryAdmissionDenial::from_authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(denial), query.name(),
                ),
            ))?;
        let (authorization, authorization_work) =
            self.observe_application_query_access(graph_work, query, access, &security)?;
        let authorization_work = authorization_work.with_capability_authorization(
            governance.authorization(),
            governance.authorization_canonical_work(),
        );
        if !authorization.belongs_to_session(graph_work.identity())
            || !governance.authorization_belongs_to_session(graph_work.identity())
        {
            return Err(graph_work_denial(query.name()));
        }
        let retained_fact_count = authorization.exact_fact_count()
            + governance
                .authorization()
                .map_or(0, |capability| capability.exact_fact_count());
        graph_work.set_retained_decision_facts(retained_fact_count);
        Ok(AdmittedApplicationQueryAuthorities {
            authorization,
            authorization_work,
            governance,
        })
    }
}
