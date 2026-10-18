//! One fresh disclosure and native phase seam serves both assessment entrances.
use super::*;
pub(super) fn advance_assessment_in_advancement<S, D>(
    runtime:&worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    admitted: &mut WorthQueryAdmittedOutputDemand<S, Family<S, D>>,
    demand: &D,
    controls: WorthQueryOutputDemandControls,
    required: &RequiredWorkflowAssessment,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> Result<
    WorthQueryWorkflowAssessmentDemandProgress<SourceQuery<S, D>>,
    WorthQueryApplicationOutputDemandDenial,
>
where
    S: ApplicationSchema + 'static,
    D: WorthQueryApplicationOutputDemand<S>,
    Family<S, D>: WorthQueryWorkflowAssessmentOutputFamily<S>,
    SourceValue<S, D>: 'static + WorthQueryApplicationProjection<S, SourceQuery<S, D>> + Clone,
    SourceQuery<S, D>: 'static,
    <SourceBinding<S, D> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = SourceBinding<S, D>>,
    <SourceBinding<S, D> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <SourceBinding<S, D> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
{
    if !std::ptr::eq(runtime, request.application) || request.branch != branch {
        return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
    }
    let _execution = phase
        .checked_execution_request_for(runtime.product_runtime())
        .map_err(WorthQueryApplicationOutputDemandDenial::advancement)?;
    for _ in 0..controls
        .resolve(runtime.output_demand_resource_profile())
        .settlement_attempts()
    {
        let disclosure = request
            .query(demand.source_intent())
            .execute_in_advancement(phase)
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
            .into_output_demand_source();
        let posture = (disclosure.rows().len() == 1)
            .then(|| Family::<S, D>::assessment_posture(&disclosure.rows()[0]));
        let progress = worth_query_execution::publication_boundary::program_publication_access()
            .advance_workflow_assessment_output(
                runtime,
                phase,
                admitted,
                request.principal,
                request.scope,
                request.branch,
                disclosure,
            )
            .map_err(map_progress_denial)?;
        if let WorthQueryOutputDemandAdvance::Settled(retained) = progress {
            let admitted = &*admitted;
            let settlement = WorthQueryApplicationOutputDemandSettlement::new(
                retained,
                admitted.observed_source().clone(),
                admitted.checkpoint_readmission_work_units(),
                admitted.checkpoint_readmission_work_bound(),
                admitted.checkpoint_readmission_charged_preparation_bytes(),
            );
            return Ok(WorthQueryWorkflowAssessmentDemandProgress::Settled(
                WorthQueryWorkflowAssessmentDemandSettlement {
                    required: required.clone(),
                    posture: posture.expect("native settled demand has one exact source"),
                    settlement,
                },
            ));
        }
    }
    Ok(WorthQueryWorkflowAssessmentDemandProgress::Pending)
}
