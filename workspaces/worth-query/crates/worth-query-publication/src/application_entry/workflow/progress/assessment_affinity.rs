use worth_query_execution::facade::workflow_advance::RequiredWorkflowAssessment;

pub(super) fn same_requirement(
    left: &RequiredWorkflowAssessment,
    right: &RequiredWorkflowAssessment,
) -> bool {
    left.instance() == right.instance()
        && left.node_path() == right.node_path()
        && left.transition_identity() == right.transition_identity()
        && left.occurrence() == right.occurrence()
        && left.query() == right.query()
        && left.parameter_type() == right.parameter_type()
        && left.result_type() == right.result_type()
        && left.binding() == right.binding()
}
