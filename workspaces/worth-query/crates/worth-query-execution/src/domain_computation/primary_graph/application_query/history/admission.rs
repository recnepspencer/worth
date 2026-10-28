//! A second genuine governance decision on the existing SubjectRead session.
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_query::{
        ApplicationHistoryMetadataFacet, ApplicationHistoryMetadataRef,
        ApplicationHistoryQueryBinding,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;

use super::super::{
    admission::governance_denial,
    disclosure::{
        admit_application_query_governance, WorthQueryApplicationGovernanceBinding,
        WorthQueryApplicationQueryGovernance, WorthQueryPendingApplicationQueryGovernance,
    },
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
};
use super::{denial::WorthQueryApplicationHistoryDenial as Denial, SubjectPlan};
use crate::domain_computation::{
    authorization::application_disclosure::contract::compile_history_disclosure_contract,
    primary_graph::{
        WorthQueryOperationAuthorizationDenial, WorthQueryPrimaryGraphApplicationRuntime,
    },
};

/// Kept beside, never substituted for, the actual SubjectRead governance.
pub(in crate::domain_computation::primary_graph::application_query) struct HistoryMetadataAdmission<
    History,
> {
    governance: WorthQueryApplicationQueryGovernance,
    _history: PhantomData<fn() -> History>,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn admit_application_history_metadata<
        History: ApplicationHistoryQueryBinding<Schema>,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        plan: &mut SubjectPlan<'_, Schema, History>,
        mut pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    ) -> Result<HistoryMetadataAdmission<History>, Denial> {
        if history.binding_identity() != plan.query.binding_identity()
            || history.binding_identity() != &self.installed_schema().binding_identity()
            || history.subject_read().query().identity() != plan.query.identity()
            || plan.runtime_authority != self.runtime.authority_identity()
        {
            return Err(Denial::SubjectMismatch);
        }
        let contract = compile_history_disclosure_contract(history).map_err(|cause| {
            Denial::Query(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid,
                cause.subject(),
            ))
        })?;
        if let Some(pending) = pending.as_mut() {
            self.refresh_capability_authorization_for_graph_work(
                execution,
                pending.authorization_mut(),
                &plan.graph_work,
            )
            .map_err(Denial::Authorization)?;
        }
        let governance = admit_application_query_governance(
            contract,
            pending,
            WorthQueryApplicationGovernanceBinding::from_session(
                &plan.graph_work,
                plan.query.identity().clone(),
                *plan.parameters.identity(),
                plan.principal.principal_entity_id(),
                plan.scope.entity_id(),
            ),
        )
        .map_err(|cause| Denial::Query(governance_denial(cause, History::IDENTITY)))?;
        if !ApplicationHistoryMetadataFacet::ALL
            .into_iter()
            .all(|facet| {
                governance.is_disclosed(
                    &ApplicationHistoryMetadataRef::<History>::new(facet)
                        .meaning()
                        .slot_key(),
                )
            })
        {
            return Err(Denial::MetadataDisclosureRefused);
        }
        if !governance.authorization_belongs_to_session(plan.graph_work.identity()) {
            return Err(Denial::SubjectMismatch);
        }
        plan.authorization_work = plan.authorization_work.with_capability_authorization(
            governance.authorization(),
            governance.authorization_canonical_work(),
        );
        plan.graph_work.record_decision_facts(
            governance
                .authorization()
                .map_or(0, |decision| decision.exact_fact_count()),
        );
        Ok(HistoryMetadataAdmission {
            governance,
            _history: PhantomData,
        })
    }
}

impl<History> HistoryMetadataAdmission<History> {
    /// Revalidate the independent decision at the same current security root
    /// used by the real SubjectRead, before protected traversal or publication.
    pub(in crate::domain_computation::primary_graph::application_query) fn validate<
        Schema: ApplicationSchema,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        security: &worth_relational::facade::snapshots::SnapshotHandle,
        plan: &SubjectPlan<'_, Schema, History>,
    ) -> Result<(), Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        if !self.governance.computation_matches(
            &plan.graph_work,
            application.runtime.authority_identity(),
            plan.query.identity(),
            plan.parameters.identity(),
            plan.principal.principal_entity_id(),
            plan.scope.entity_id(),
        ) {
            return Err(Denial::SubjectMismatch);
        }
        if let Some(authorization) = self.governance.authorization() {
            authorization
                .validate_currentness_in(
                    execution,
                    runtime,
                    security,
                    application.authorization.bridge(),
                )
                .map_err(|cause| {
                    Denial::Authorization(WorthQueryOperationAuthorizationDenial::new(
                        cause,
                        History::IDENTITY,
                    ))
                })?;
        }
        Ok(())
    }
}
