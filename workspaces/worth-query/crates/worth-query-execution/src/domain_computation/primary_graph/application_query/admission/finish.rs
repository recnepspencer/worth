use worth_query_admission::facade::{
    application_query::WorthQueryAdmittedApplicationQueryParameters,
    graph_obligation::{WorthQueryAdmittedGraphWorkPlan, WorthQueryGraphWorkIntent},
};
use worth_query_admission::integration::{
    admit_application_query_graph_work, derive_graph_read_access_requirements_for_contract,
    review_application_query_graph_work, select_installed_graph_obligations,
    WorthQueryReviewedApplicationQueryGraphWork,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkPhases, WorthQueryInstalledApplicationQuery,
    WorthQueryInstalledGraphObligationSetIdentity,
};
use worth_relational::facade::indexes::DerivedIndexId;

use super::{
    denial::{denial, graph_work_denial},
    work_limit::{application_query_graph_read_budget, validate_work_limit},
};
use crate::domain_computation::primary_graph::application_query::{
    admission_preparation::validate_admission_request,
    basis::admit_application_query_basis,
    disclosure::{
        compile_disclosure_contract, WorthQueryAdmittedApplicationDisclosureContract,
        WorthQueryPendingApplicationQueryGovernance,
    },
    execution_shape::validate_one_shot_shape,
    runtime_support::primary_graph_support_inventory,
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraph, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkAccessContextAffinity, WorthQueryManagedGraphWorkSession,
};

struct PreparedApplicationQueryGraphWork {
    plan: WorthQueryAdmittedGraphWorkPlan,
    obligation_identity: WorthQueryInstalledGraphObligationSetIdentity,
    disclosure: WorthQueryAdmittedApplicationDisclosureContract,
    canonical_work: WorthQueryCanonicalWorkPhases,
    continuation_index_id: Option<DerivedIndexId>,
}

struct ReviewedApplicationQueryGraphWork {
    work: WorthQueryReviewedApplicationQueryGraphWork,
    obligation_identity: WorthQueryInstalledGraphObligationSetIdentity,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph::application_query) fn finish_application_query_admission<
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
        parameters: WorthQueryAdmittedApplicationQueryParameters,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
        pending_governance: Option<WorthQueryPendingApplicationQueryGovernance>,
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
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            )
        })?;
        let prepared =
            self.prepare_application_query_graph_work(query, &parameters, &controls, graph)?;
        validate_admission_request(controls.request_scope(), query.name())?;
        let (basis_selection, controls) = controls.into_admission_parts();
        let basis = admit_application_query_basis(self, basis_selection)?;
        validate_admission_request(controls.request_scope(), query.name())?;
        let mut graph_work = self.start_application_query_graph_work(
            prepared.plan,
            &prepared.obligation_identity,
            query,
            access,
            pending_governance.as_ref(),
            basis.identity(),
            graph,
        )?;
        let authorities = self.admit_application_query_authorities(
            &mut graph_work,
            &basis,
            query,
            access,
            &parameters,
            pending_governance,
            prepared.disclosure,
        )?;
        Ok(WorthQueryAdmittedApplicationQueryPlan {
            runtime_authority: self.runtime.authority_identity(),
            graph_authority_identity: self
                .primary_graph_authority
                .authority_identity()
                .to_string(),
            provider_identity: self.primary_graph_authority.provider_identity().to_string(),
            query,
            principal: access.principal(),
            scope: access.scope(),
            parameters,
            controls,
            canonical_work: prepared.canonical_work,
            continuation_index_id: prepared.continuation_index_id,
            continuation_state: None,
            basis,
            graph_work,
            authorization: authorities.authorization,
            authorization_work: authorities.authorization_work,
            governance: authorities.governance,
        })
    }

    fn prepare_application_query_graph_work<Query, Parameters, QueryResult, Scope>(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        parameters: &WorthQueryAdmittedApplicationQueryParameters,
        controls: &WorthQueryApplicationQueryControls<'_, Schema>,
        graph: &WorthQueryPrimaryGraph,
    ) -> Result<PreparedApplicationQueryGraphWork, WorthQueryApplicationQueryAdmissionDenial> {
        let reviewed =
            self.review_application_query_graph_work(query, parameters, controls, graph)?;
        let canonical_work = WorthQueryCanonicalWorkPhases::new(
            query.installation_canonical_work(),
            parameters
                .canonical_basis()
                .work()
                .combine(reviewed.work.review().requirements().canonical_work()),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
        );
        validate_work_limit(reviewed.work.review(), controls, query.name())?;
        if let Some(denial) = reviewed.work.review().denial() {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::GraphReadPlan(denial.kind()),
                query.name(),
            ));
        }
        let plan =
            admit_application_query_graph_work(reviewed.work, &self.graph_work_resource_support())
                .map_err(|_| graph_work_denial(query.name()))?;
        validate_one_shot_shape(query)?;
        let disclosure = compile_disclosure_contract(query, &graph.layout).map_err(|denial| {
            WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid,
                denial.subject(),
            )
        })?;
        let continuation_index_id = continuation_index_id(query, controls, graph)?;
        Ok(PreparedApplicationQueryGraphWork {
            plan,
            obligation_identity: reviewed.obligation_identity,
            disclosure,
            canonical_work,
            continuation_index_id,
        })
    }

    fn review_application_query_graph_work<Query, Parameters, QueryResult, Scope>(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        parameters: &WorthQueryAdmittedApplicationQueryParameters,
        controls: &WorthQueryApplicationQueryControls<'_, Schema>,
        graph: &WorthQueryPrimaryGraph,
    ) -> Result<ReviewedApplicationQueryGraphWork, WorthQueryApplicationQueryAdmissionDenial> {
        let requirements = derive_graph_read_access_requirements_for_contract(
            query.read_family_binding().planning_contract(),
            controls.lane(),
            controls.maximum_result_count().get(),
            parameters.identity(),
            query.canonical_work_policy().admission_planning(),
        )
        .map_err(|_| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::CanonicalWorkDenied,
                query.name(),
            )
        })?;
        let inventory = primary_graph_support_inventory(
            &graph.layout,
            query.continuation(),
            query.live(),
            &requirements,
        );
        let obligations = query.retain_graph_obligations_for_admission();
        let obligation_identity = obligations.identity().clone();
        let selected = select_installed_graph_obligations(
            obligations,
            WorthQueryGraphWorkIntent::application_query_read(),
        )
        .map_err(|_| graph_work_denial(query.name()))?;
        let work = review_application_query_graph_work(
            selected,
            requirements,
            inventory,
            application_query_graph_read_budget(
                self.runtime.application_query_resource_profile(),
                controls,
            ),
        )
        .map_err(|_| graph_work_denial(query.name()))?;
        Ok(ReviewedApplicationQueryGraphWork {
            work,
            obligation_identity,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn start_application_query_graph_work<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        plan: WorthQueryAdmittedGraphWorkPlan,
        obligation_identity: &WorthQueryInstalledGraphObligationSetIdentity,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        pending_governance: Option<&WorthQueryPendingApplicationQueryGovernance>,
        basis_identity: &super::super::WorthQueryApplicationBasisIdentity,
        graph: &WorthQueryPrimaryGraph,
    ) -> Result<WorthQueryManagedGraphWorkSession, WorthQueryApplicationQueryAdmissionDenial> {
        let capability_identity = pending_governance
            .map(WorthQueryPendingApplicationQueryGovernance::installed_capability_identity);
        let affinity = capability_identity.map_or_else(
            || WorthQueryGraphWorkAccessContextAffinity::entity(access.scope().entity_id()),
            |identity| {
                WorthQueryGraphWorkAccessContextAffinity::governed_entity(
                    access.scope().entity_id(),
                    identity,
                )
            },
        );
        WorthQueryManagedGraphWorkSession::start_query(
            plan,
            self.runtime.authority_identity(),
            query.binding_identity(),
            obligation_identity,
            query.authority_identity(),
            access.principal().principal_entity_id(),
            affinity,
            basis_identity,
            self.graph_work_provider_identity(),
            graph.query_session_port(),
        )
        .map_err(|_| graph_work_denial(query.name()))
    }
}

fn continuation_index_id<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    controls: &WorthQueryApplicationQueryControls<'_, Schema>,
    graph: &WorthQueryPrimaryGraph,
) -> Result<Option<DerivedIndexId>, WorthQueryApplicationQueryAdmissionDenial>
where
    Schema: ApplicationSchema,
{
    use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;

    if controls.lane() != WorthQueryApplicationQueryLane::Continuation {
        return Ok(None);
    }
    query
        .continuation()
        .and_then(|contract| graph.layout.continuation_ordering_index_id(contract))
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                query.name(),
            )
        })
        .map(Some)
}
