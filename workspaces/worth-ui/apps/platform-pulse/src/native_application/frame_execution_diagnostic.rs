use worth_ui::facade::app::UiMountedFrameOutcome;

pub(super) fn outcome_label(outcome: &UiMountedFrameOutcome) -> String {
    match outcome {
        UiMountedFrameOutcome::Published(_) => "published".to_owned(),
        UiMountedFrameOutcome::Unchanged(_) => "unchanged".to_owned(),
        UiMountedFrameOutcome::Reconciled(_) => "reconciled".to_owned(),
        UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => format!(
            "rejected-before-effects:{:?}",
            rejected
                .rejections()
                .iter()
                .map(|rejection| rejection.denial())
                .collect::<Vec<_>>()
        ),
        UiMountedFrameOutcome::InFlight(_) => "in-flight".to_owned(),
        UiMountedFrameOutcome::PresentationIndeterminate(frame) => format!(
            "presentation-indeterminate:physical-recovery={}",
            frame.report().awaits_physical_recovery()
        ),
        UiMountedFrameOutcome::Superseded(_) => "superseded".to_owned(),
        UiMountedFrameOutcome::RetentionDenied(_) => "retention-denied".to_owned(),
        UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            format!("admission-denied:{:?}", rejection.denial())
        }
        UiMountedFrameOutcome::CompletionDenied(_) => "completion-denied".to_owned(),
    }
}

pub(super) fn stop_label(
    stop: &worth_ui::facade::app::WorthUiMountedFrameExecutionStop<'_>,
) -> String {
    match stop {
        worth_ui::facade::app::WorthUiMountedFrameExecutionStop::PublicationLease(_) => {
            "publication-lease".to_owned()
        }
        worth_ui::facade::app::WorthUiMountedFrameExecutionStop::HostMeasurement(denial) => {
            format!("host-measurement:{denial:?}")
        }
        worth_ui::facade::app::WorthUiMountedFrameExecutionStop::HostMeasurementTransition(
            denial,
        ) => host_measurement_transition_label(denial),
        worth_ui::facade::app::WorthUiMountedFrameExecutionStop::OccurrenceGeometry(denial) => {
            format!("occurrence-geometry:{denial:?}")
        }
        worth_ui::facade::app::WorthUiMountedFrameExecutionStop::FrameworkTransition(_) => {
            "framework-transition".to_owned()
        }
        worth_ui::facade::app::WorthUiMountedFrameExecutionStop::Preparation(denial) => {
            format!("preparation:{denial:?}")
        }
    }
}

fn host_measurement_transition_label(
    denial: &worth_ui::facade::app::UiMountedHostMeasurementTransitionDenial,
) -> String {
    use worth_ui::facade::app::UiMountedHostMeasurementTransitionDenial as Denial;
    use worth_ui::facade::app::UiMountedHostMeasurementUnexpectedTransition as Unexpected;

    match denial {
        Denial::AllocationReplanDenied(_) => "host-measurement:allocation-replan-denied".to_owned(),
        Denial::ViewportResizeDenied(_) => "host-measurement:viewport-resize-denied".to_owned(),
        Denial::AllocationReplanSelectionDenied(_) => {
            "host-measurement:allocation-replan-selection-denied".to_owned()
        }
        Denial::AllocationFrameResolutionDenied(_) => {
            "host-measurement:allocation-frame-resolution-denied".to_owned()
        }
        Denial::AllocationInvalidationNarrowingDenied(detail) => {
            format!("host-measurement:allocation-invalidation-narrowing-denied:{detail:?}")
        }
        Denial::FrameworkTransitionPlanningDenied(_) => {
            "host-measurement:framework-transition-planning-denied".to_owned()
        }
        Denial::FrameworkTransitionExecutionDenied(_) => {
            "host-measurement:framework-transition-execution-denied".to_owned()
        }
        Denial::DispatcherDenied { .. } => "host-measurement:dispatcher-denied".to_owned(),
        Denial::UnexpectedSuccessfulTransition(transition) => match transition {
            Unexpected::ReadyToExecute => "host-measurement:unexpected-ready-to-execute".to_owned(),
            Unexpected::ResizePreviewPublished => {
                "host-measurement:unexpected-resize-preview-published".to_owned()
            }
            Unexpected::DurableResizeCommitted => {
                "host-measurement:unexpected-durable-resize-committed".to_owned()
            }
            Unexpected::DragResizePreviewPending => {
                "host-measurement:unexpected-drag-resize-preview-pending".to_owned()
            }
        },
    }
}
