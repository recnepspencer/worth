use super::{
    WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageExecutionContext,
    WorthQueryWorkflowStageWorkspace, WorthQueryWorkflowValue,
};

/// An owner-only application slot issued while consuming a canonical batch.
/// No caller can build a single-result application or choose its context.
///
/// ```
/// use worth_query::facade::domain::*;
/// fn forge(
///     input: WorthQueryWorkflowValue, computed: WorthQueryWorkflowStageComputePayload,
///     context: &WorthQueryWorkflowStageExecutionContext<'_>,
///     workspace: &mut WorthQueryWorkflowStageWorkspace<'_>,
/// ) { let _ = (input, computed, context, workspace); }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::*;
/// fn forge(
///     input: WorthQueryWorkflowValue, computed: WorthQueryWorkflowStageComputePayload,
///     context: &WorthQueryWorkflowStageExecutionContext<'_>,
///     workspace: &mut WorthQueryWorkflowStageWorkspace<'_>,
/// ) { let _ = WorthQueryWorkflowStageApplication::new(input, computed, context, workspace); }
/// ```
pub struct WorthQueryWorkflowStageApplication<'apply, 'context, 'workspace> {
    input: WorthQueryWorkflowValue,
    computed: WorthQueryWorkflowStageComputePayload,
    context: &'apply WorthQueryWorkflowStageExecutionContext<'context>,
    workspace: &'apply mut WorthQueryWorkflowStageWorkspace<'workspace>,
}

impl<'apply, 'context, 'workspace>
    WorthQueryWorkflowStageApplication<'apply, 'context, 'workspace>
{
    pub(in crate::domain_installation::operation_execution) fn new(
        input: WorthQueryWorkflowValue,
        computed: WorthQueryWorkflowStageComputePayload,
        context: &'apply WorthQueryWorkflowStageExecutionContext<'context>,
        workspace: &'apply mut WorthQueryWorkflowStageWorkspace<'workspace>,
    ) -> Self {
        Self {
            input,
            computed,
            context,
            workspace,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryWorkflowValue,
        WorthQueryWorkflowStageComputePayload,
        &'apply WorthQueryWorkflowStageExecutionContext<'context>,
        &'apply mut WorthQueryWorkflowStageWorkspace<'workspace>,
    ) {
        (self.input, self.computed, self.context, self.workspace)
    }
}
