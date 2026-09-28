use super::{
    UiPendingMountedPreviewTransition, WorthUiFrameworkTurnCompletion,
    WorthUiFrameworkTurnExecution,
};

impl<'runtime> WorthUiFrameworkTurnCompletion<'runtime> {
    pub fn planning_counters(&self) -> Option<super::super::UiFrameworkTransitionPlanningCounters> {
        match self {
            Self::ReadyToExecute { execution } => Some(execution.planning_counters()),
            Self::AllocationInvalidationsNarrowed {
                planning_counters, ..
            }
            | Self::ViewportResizeResolved {
                planning_counters, ..
            }
            | Self::ViewportResizeDenied {
                planning_counters, ..
            }
            | Self::ResizePreviewPublished {
                planning_counters, ..
            }
            | Self::DurableResizeCommitted {
                planning_counters, ..
            }
            | Self::DragResizePreviewPending {
                planning_counters, ..
            } => Some(*planning_counters),
            Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. } => None,
        }
    }

    pub fn into_execution(self) -> Result<WorthUiFrameworkTurnExecution<'runtime>, Box<Self>> {
        match self {
            Self::ReadyToExecute { execution } => Ok(execution),
            other @ (Self::AllocationInvalidationsNarrowed { .. }
            | Self::ViewportResizeResolved { .. }
            | Self::ViewportResizeDenied { .. }
            | Self::ResizePreviewPublished { .. }
            | Self::DurableResizeCommitted { .. }
            | Self::DragResizePreviewPending { .. }
            | Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. }) => Err(Box::new(other)),
        }
    }
    pub fn narrowed_plan(&self) -> Option<&crate::runtime::UiNarrowedAllocationFramePlan> {
        match self {
            Self::AllocationInvalidationsNarrowed { plan, .. } => Some(plan),
            Self::ReadyToExecute { .. }
            | Self::ViewportResizeResolved { .. }
            | Self::ViewportResizeDenied { .. }
            | Self::ResizePreviewPublished { .. }
            | Self::DurableResizeCommitted { .. }
            | Self::DragResizePreviewPending { .. }
            | Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. } => None,
        }
    }
    pub fn replan_selection(&self) -> Option<&crate::graph::UiAdmittedReplanNeighborhoodSet> {
        match self {
            Self::AllocationInvalidationsNarrowed { selection, .. }
            | Self::DurableResizeCommitted { selection, .. } => Some(selection),
            Self::DragResizePreviewPending { durable, .. } => Some(&durable.selection),
            Self::ReadyToExecute { .. }
            | Self::ViewportResizeResolved { .. }
            | Self::ViewportResizeDenied { .. }
            | Self::ResizePreviewPublished { .. }
            | Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. } => None,
        }
    }
    pub fn replan_transaction(
        &self,
    ) -> Option<&crate::runtime::UiAllocationReplanTransactionOutcome> {
        match self {
            Self::AllocationInvalidationsNarrowed { transaction, .. } => Some(transaction),
            Self::ReadyToExecute { .. }
            | Self::ViewportResizeResolved { .. }
            | Self::ViewportResizeDenied { .. }
            | Self::ResizePreviewPublished { .. }
            | Self::DurableResizeCommitted { .. }
            | Self::DragResizePreviewPending { .. }
            | Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. } => None,
        }
    }
    pub fn denied_replan_inspection(
        &self,
    ) -> Option<worth_ui_inspection::UiAllocationInspectionDeniedAttempt> {
        let Self::AllocationInvalidationsNarrowed {
            plan,
            selection,
            transaction: crate::runtime::UiAllocationReplanTransactionOutcome::Denied(denial),
            ..
        } = self
        else {
            return None;
        };
        Some(crate::evidence::project_denied_replan_inspection(
            plan, selection, denial,
        ))
    }
    pub fn viewport_resize_outcome(&self) -> Option<&crate::runtime::UiViewportResizeOutcome> {
        match self {
            Self::ViewportResizeResolved { outcome, .. } => Some(outcome),
            Self::ReadyToExecute { .. }
            | Self::AllocationInvalidationsNarrowed { .. }
            | Self::ViewportResizeDenied { .. }
            | Self::ResizePreviewPublished { .. }
            | Self::DurableResizeCommitted { .. }
            | Self::DragResizePreviewPending { .. }
            | Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. } => None,
        }
    }
    pub fn durable_resize_outcome(&self) -> Option<&crate::runtime::UiDurableResizeCommitOutcome> {
        match self {
            Self::DurableResizeCommitted { outcome, .. } => Some(outcome),
            Self::ReadyToExecute { .. }
            | Self::AllocationInvalidationsNarrowed { .. }
            | Self::ViewportResizeResolved { .. }
            | Self::ViewportResizeDenied { .. }
            | Self::ResizePreviewPublished { .. }
            | Self::DragResizePreviewPending { .. }
            | Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. } => None,
        }
    }
    pub(crate) fn into_pending_mounted_preview(
        self,
    ) -> Result<
        (
            UiPendingMountedPreviewTransition<'runtime>,
            super::super::UiFrameworkTransitionPlanningCounters,
        ),
        Box<Self>,
    > {
        match self {
            Self::ResizePreviewPublished {
                pending,
                planning_counters,
            } => Ok((
                UiPendingMountedPreviewTransition::PreviewOnly { preview: pending },
                planning_counters,
            )),
            Self::DragResizePreviewPending {
                preview,
                durable,
                planning_counters,
            } => Ok((
                UiPendingMountedPreviewTransition::DragResize {
                    preview,
                    durable: Box::new(durable),
                },
                planning_counters,
            )),
            other @ (Self::ReadyToExecute { .. }
            | Self::AllocationInvalidationsNarrowed { .. }
            | Self::ViewportResizeResolved { .. }
            | Self::ViewportResizeDenied { .. }
            | Self::DurableResizeCommitted { .. }
            | Self::AllocationReplanSelectionDenied { .. }
            | Self::AllocationFrameResolutionDenied { .. }
            | Self::AllocationInvalidationNarrowingDenied { .. }
            | Self::FrameworkTransitionPlanningDenied { .. }
            | Self::FrameworkTransitionExecutionDenied { .. }
            | Self::Denied { .. }) => Err(Box::new(other)),
        }
    }
}
