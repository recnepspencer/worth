use super::WorthUiMountedPreviewFollowOn;

impl WorthUiMountedPreviewFollowOn {
    pub fn durable_resize_outcome(&self) -> Option<&crate::runtime::UiDurableResizeCommitOutcome> {
        match self {
            Self::PreviewOnly => None,
            Self::DurableResizeCommitted { outcome, .. } => Some(outcome.as_ref()),
            Self::DurableResizeDenied { .. } => None,
            Self::DurableResizeSuppressedByPreviewIsolation { .. } => None,
        }
    }

    pub fn durable_resize_denial(
        &self,
    ) -> Option<&crate::runtime::UiDurableResizeCommitDenialReport> {
        match self {
            Self::DurableResizeDenied { report, .. } => Some(report.as_ref()),
            Self::PreviewOnly
            | Self::DurableResizeCommitted { .. }
            | Self::DurableResizeSuppressedByPreviewIsolation { .. } => None,
        }
    }

    pub fn replan_selection(&self) -> Option<&crate::graph::UiAdmittedReplanNeighborhoodSet> {
        match self {
            Self::PreviewOnly => None,
            Self::DurableResizeCommitted { selection, .. } => Some(selection.as_ref()),
            Self::DurableResizeDenied { selection, .. } => Some(selection.as_ref()),
            Self::DurableResizeSuppressedByPreviewIsolation { selection, .. } => {
                Some(selection.as_ref())
            }
        }
    }
}
