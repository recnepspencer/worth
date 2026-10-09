use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryMarkerIdentity,
        ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
        WorthQueryWorkflowAssessmentOutputFamily, WorthQueryWorkflowAssessmentPosture,
    },
    application_installation::WorthQueryWorkflowVocabulary,
    primary_graph::{
        WorthQueryAdmittedOutputDemand, WorthQueryApplicationProjection,
        WorthQueryOutputDemandAdvance, WorthQueryOutputDemandNotifications,
    },
};
use worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowAssessment;
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandRequest,
    WorthQueryApplicationOutputDemandSettlement, WorthQueryApplicationRequest,
    WorthQueryOutputDemandControls,
};

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type SourceBinding<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> = <<SourceBinding<Schema, Demand> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// The kind of a `WorthQueryWorkflowAssessmentDemandPreparationDenial`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowAssessmentDemandPreparationDenialKind {
    NotAwaitingAssessment,
    ContractMismatch,
    RequirementMismatch,
}

/// Why an assessment demand did not start: the instance is not awaiting an assessment, the
/// demand does not match the installed contract, or it does not match the requirement.
#[derive(Debug)]
pub struct WorthQueryWorkflowAssessmentDemandPreparationDenial {
    kind: WorthQueryWorkflowAssessmentDemandPreparationDenialKind,
}

impl WorthQueryWorkflowAssessmentDemandPreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowAssessmentDemandPreparationDenialKind {
        self.kind
    }

    pub(super) const fn not_assessment() -> Self {
        Self {
            kind: WorthQueryWorkflowAssessmentDemandPreparationDenialKind::NotAwaitingAssessment,
        }
    }

    const fn contract_mismatch() -> Self {
        Self {
            kind: WorthQueryWorkflowAssessmentDemandPreparationDenialKind::ContractMismatch,
        }
    }

    pub(super) const fn requirement_mismatch() -> Self {
        Self {
            kind: WorthQueryWorkflowAssessmentDemandPreparationDenialKind::RequirementMismatch,
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowAssessmentDemandPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow assessment demand denied: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for WorthQueryWorkflowAssessmentDemandPreparationDenial {}

/// A demand for the assessment a workflow instance awaits. `start` admits it.
pub struct WorthQueryWorkflowAssessmentDemandRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Spec,
    Demand,
> where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    required: RequiredWorkflowAssessment,
    workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec>,
    demand:
        WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>,
}

impl<'application, 'principal, 'scope, Schema, Spec, Demand>
    WorthQueryWorkflowAssessmentDemandRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Demand,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub(super) fn new(
        application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec>,
        required: RequiredWorkflowAssessment,
        demand: Demand,
    ) -> Result<Self, WorthQueryWorkflowAssessmentDemandPreparationDenial> {
        if !matches_contract::<Schema, Demand>(&required) {
            return Err(WorthQueryWorkflowAssessmentDemandPreparationDenial::contract_mismatch());
        }
        Ok(Self {
            required,
            workflow,
            demand: WorthQueryApplicationOutputDemandRequest::new(
                application,
                principal,
                scope,
                branch,
                demand,
            ),
        })
    }

    pub const fn required(&self) -> &RequiredWorkflowAssessment {
        &self.required
    }

    pub fn controls(mut self, controls: WorthQueryOutputDemandControls) -> Self {
        self.demand = self.demand.controls(controls);
        self
    }
}

impl<'application, 'principal, 'scope, Schema, Spec, Demand>
    WorthQueryWorkflowAssessmentDemandRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Demand,
    >
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    Family<Schema, Demand>: WorthQueryWorkflowAssessmentOutputFamily<Schema>,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn start(
        self,
    ) -> Result<
        WorthQueryWorkflowAssessmentDemandHandle<'application, Schema, Spec, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let (admitted, demand, controls) = self.demand.start_for_workflow(self.workflow)?;
        Ok(WorthQueryWorkflowAssessmentDemandHandle {
            required: self.required,
            workflow: self.workflow,
            admitted,
            demand,
            controls,
        })
    }
}

/// An admitted assessment demand. `settle` drives it; `close` releases this observer only.
pub struct WorthQueryWorkflowAssessmentDemandHandle<'application, Schema, Spec, Demand>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    required: RequiredWorkflowAssessment,
    workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec>,
    admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    demand: Demand,
    controls: WorthQueryOutputDemandControls,
}

/// Whether an assessment demand has settled yet.
pub enum WorthQueryWorkflowAssessmentDemandProgress<Query> {
    Pending,
    Settled(WorthQueryWorkflowAssessmentDemandSettlement<Query>),
}

/// A settled assessment demand: the requirement, its posture, and the output settlement to
/// accept with `accept_assessment`.
pub struct WorthQueryWorkflowAssessmentDemandSettlement<Query> {
    required: RequiredWorkflowAssessment,
    posture: WorthQueryWorkflowAssessmentPosture,
    settlement: WorthQueryApplicationOutputDemandSettlement<Query>,
}

impl<Query> WorthQueryWorkflowAssessmentDemandSettlement<Query> {
    pub const fn required(&self) -> &RequiredWorkflowAssessment {
        &self.required
    }
    pub const fn posture(&self) -> WorthQueryWorkflowAssessmentPosture {
        self.posture
    }
    pub const fn settlement(&self) -> &WorthQueryApplicationOutputDemandSettlement<Query> {
        &self.settlement
    }

    pub(super) const fn owner_settlement(
        &self,
    ) -> &WorthQueryApplicationOutputDemandSettlement<Query> {
        &self.settlement
    }
}

impl<Schema, Spec, Demand> WorthQueryWorkflowAssessmentDemandHandle<'_, Schema, Spec, Demand>
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    Family<Schema, Demand>: WorthQueryWorkflowAssessmentOutputFamily<Schema>,
    SourceValue<Schema, Demand>:
        'static + WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
    SourceQuery<Schema, Demand>: 'static,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub const fn required(&self) -> &RequiredWorkflowAssessment {
        &self.required
    }

    pub fn settle(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryWorkflowAssessmentDemandProgress<SourceQuery<Schema, Demand>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        self.workflow
            .runtime()
            .with_application_advancement(fresh_request.scope, |phase| {
                for _ in 0..self
                    .controls
                    .resolve(self.workflow.runtime().output_demand_resource_profile())
                    .settlement_attempts()
                {
                    let disclosure = fresh_request
                        .query(self.demand.source_intent())
                        .execute_in_advancement(&phase)
                        .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
                        .into_output_demand_source();
                    let posture = (disclosure.rows().len() == 1).then(|| {
                        Family::<Schema, Demand>::assessment_posture(&disclosure.rows()[0])
                    });
                    let progress = self
                .workflow
                .advance_workflow_assessment_output(
                    &phase,
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    &mut self.admitted,
                    fresh_request.principal,
                    fresh_request.scope,
                    fresh_request.branch,
                    disclosure,
                )
                .map_err(map_progress_denial)?;
                    if let WorthQueryOutputDemandAdvance::Settled(retained) = progress {
                        let settlement = WorthQueryApplicationOutputDemandSettlement::new(
                            retained,
                            self.admitted.observed_source().clone(),
                        );
                        return Ok(WorthQueryWorkflowAssessmentDemandProgress::Settled(
                            WorthQueryWorkflowAssessmentDemandSettlement {
                                required: self.required.clone(),
                                posture: posture.expect(
                                    "a settled output demand has one exact typed source row",
                                ),
                                settlement,
                            },
                        ));
                    }
                }
                Ok(WorthQueryWorkflowAssessmentDemandProgress::Pending)
            })
            .map_err(WorthQueryApplicationOutputDemandDenial::advancement)?
    }

    pub fn close(&mut self) {
        self.admitted.close();
    }

    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryApplicationOutputDemandDenial> {
        self.admitted
            .notifications()
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)
    }
}

fn map_progress_denial(
    denial: worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial,
) -> WorthQueryApplicationOutputDemandDenial {
    if denial.kind() == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded {
        WorthQueryApplicationOutputDemandDenial::Superseded
    } else {
        WorthQueryApplicationOutputDemandDenial::Demand(denial)
    }
}

fn matches_contract<Schema, Demand>(required: &RequiredWorkflowAssessment) -> bool
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    required.query() == SourceQuery::<Schema, Demand>::IDENTIFIER
        && required.parameter_type()
            == SourceQuery::<Schema, Demand>::PARAMETER_TYPE_IDENTITY.as_str()
        && required.result_type() == SourceQuery::<Schema, Demand>::RESULT_TYPE_IDENTITY.as_str()
        && required.binding() == SourceBinding::<Schema, Demand>::IDENTITY
}
