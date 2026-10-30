pub use worth_query::facade::domain;

pub use super::{GeometryDomain, ReadFamily, WorkflowRead};

pub struct WorkflowParallelProvider;
pub struct SerialParallelProvider;

impl domain::WorthQueryWorkflowParallelAdmissionProvider<GeometryDomain, WorkflowRead, ReadFamily>
    for WorkflowParallelProvider
{
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn admit_parallel_frontier(
        &self,
        call: &domain::WorthQueryWorkflowParallelAdmissionCall,
    ) -> Result<
        worth_signal::facade::adapters::FrontierRouteEvidenceReceipt,
        domain::WorthQueryWorkflowParallelAdmissionFailure,
    > {
        assert_eq!(call.execution_resources().strategy(), "fixture-bounded");
        assert_eq!(
            call.resource_envelope().cancellation_safe_point().as_str(),
            "fixture-chunk-boundary"
        );
        let frontier = call.frontier();
        if frontier
            .iter()
            .map(|stage| stage.stage_identity())
            .collect::<Vec<_>>()
            != ["left", "right"]
            || frontier.iter().any(|stage| {
                !stage.graph_read_roles().is_empty()
                    || !stage.touch_roles().is_empty()
                    || !stage.effect_roles().is_empty()
            })
        {
            return Err(domain::WorthQueryWorkflowParallelAdmissionFailure::new(
                "fixture admits only the installed pure left/right frontier",
            ));
        }
        Ok(
            worth_signal::facade::adapters::FrontierRouteEvidenceReceipt::from_reason(
                worth_signal::facade::adapters::FrontierRouteEvidenceReason::AdmittedThroughput,
            ),
        )
    }
}

impl domain::WorthQueryWorkflowParallelAdmissionProvider<GeometryDomain, WorkflowRead, ReadFamily>
    for SerialParallelProvider
{
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn admit_parallel_frontier(
        &self,
        _call: &domain::WorthQueryWorkflowParallelAdmissionCall,
    ) -> Result<
        worth_signal::facade::adapters::FrontierRouteEvidenceReceipt,
        domain::WorthQueryWorkflowParallelAdmissionFailure,
    > {
        Ok(
            worth_signal::facade::adapters::FrontierRouteEvidenceReceipt::from_reason(
                worth_signal::facade::adapters::FrontierRouteEvidenceReason::BelowMinStageWidth,
            ),
        )
    }
}
