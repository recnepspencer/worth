//! A second genuine governance decision on the existing SubjectRead session.

use crate::domain_computation::authorization::CapabilityRevalidationScopedOps;
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
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use super::{denial::WorthQueryApplicationHistoryDenial as Denial, SubjectPlan};
use crate::domain_computation::{
    authorization::application_disclosure::contract::compile_history_disclosure_contract,
    primary_graph::{
        WorthQueryOperationAuthorizationDenial, WorthQueryPrimaryGraphApplicationRuntime,
    },
};

/// Kept beside, never substituted for, the actual SubjectRead governance.
pub(in crate::domain_computation::primary_graph::application_query) struct HistoryMetadataAdmission
{
    governance: WorthQueryApplicationQueryGovernance,
    history: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity,
}

pub(super) trait AdmissionParentOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn admit_application_history_metadata<History: ApplicationHistoryQueryBinding<Schema>>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        plan: &mut SubjectPlan<'_, Schema, History>,
        pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    ) -> Result<HistoryMetadataAdmission, Denial> {
        if history.binding_identity() != plan.query.binding_identity()
            || history.binding_identity() != &self.installed_schema().binding_identity()
            || history.subject_read().query().identity() != plan.query.identity()
            || plan.runtime_authority != self.authority_identity()
        {
            return Err(Denial::SubjectMismatch);
        }
        let contract = compile_history_disclosure_contract(history).map_err(|cause| {
            Denial::Query(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid,
                cause.subject(),
            ))
        })?;
        admit_metadata::<History, _, _, _, _, _, _, _>(
            self.application_query_host(),
            execution,
            plan,
            contract,
            pending,
            History::IDENTITY,
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > AdmissionParentOps<Schema> for HostRuntime
{
}

pub(in crate::domain_computation::primary_graph::application_query) fn admit_metadata<
    History,
    Schema,
    Q,
    P,
    R,
    A,
    I,
    E,
>(
    application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: &mut WorthQueryAdmittedApplicationQueryPlan<'_, Schema, Q, P, R, A, I, E>,
    contract: super::super::disclosure::WorthQueryAdmittedApplicationDisclosureContract,
    mut pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    identity: &str,
) -> Result<HistoryMetadataAdmission, Denial>
where
    Schema: ApplicationSchema,
    History: worth_query_declaration::facade::portable_identity::WorthQueryPortableType,
{
    if let Some(pending) = pending.as_mut() {
        application
            .refresh_capability_authorization_for_graph_work(
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
    .map_err(|cause| Denial::Query(governance_denial(cause, identity)))?;
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
        history: History::PORTABLE_TYPE_IDENTITY,
    })
}

pub(in crate::domain_computation::primary_graph::application_query) enum MetadataValidationDenial {
    Session,
    Authorization(WorthQueryOperationAuthorizationDenial),
}

impl From<MetadataValidationDenial> for Denial {
    fn from(cause: MetadataValidationDenial) -> Self {
        match cause {
            MetadataValidationDenial::Session => Self::SubjectMismatch,
            MetadataValidationDenial::Authorization(cause) => Self::Authorization(cause),
        }
    }
}

impl From<MetadataValidationDenial>
    for super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial
{
    fn from(cause: MetadataValidationDenial) -> Self {
        match cause {
            MetadataValidationDenial::Session => Self::Session,
            MetadataValidationDenial::Authorization(cause) => Self::Authorization(cause),
        }
    }
}

impl HistoryMetadataAdmission {
    pub(in crate::domain_computation::primary_graph::application_query) fn belongs_to<
        H: worth_query_declaration::facade::portable_identity::WorthQueryPortableType,
    >(
        &self,
    ) -> bool {
        self.history == H::PORTABLE_TYPE_IDENTITY
    }
    /// Revalidate the independent decision at the same current security root
    /// used by the real SubjectRead, before protected traversal or publication.
    pub(in crate::domain_computation::primary_graph::application_query) fn validate<
        Schema: ApplicationSchema,
        Q,
        P,
        R,
        A,
        I,
        E,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        security: &worth_relational::facade::snapshots::SnapshotHandle,
        plan: &WorthQueryAdmittedApplicationQueryPlan<'_, Schema, Q, P, R, A, I, E>,
    ) -> Result<(), MetadataValidationDenial> {
        if !self.governance.computation_matches(
            &plan.graph_work,
            application.authority_identity(),
            plan.query.identity(),
            plan.parameters.identity(),
            plan.principal.principal_entity_id(),
            plan.scope.entity_id(),
        ) {
            return Err(MetadataValidationDenial::Session);
        }
        if let Some(authorization) = self.governance.authorization() {
            authorization
                .validate_currentness_in(
                    execution,
                    runtime,
                    security,
                    application.authorization().bridge(),
                )
                .map_err(|cause| {
                    MetadataValidationDenial::Authorization(
                        WorthQueryOperationAuthorizationDenial::new(cause, self.history.as_str()),
                    )
                })?;
        }
        Ok(())
    }
}
