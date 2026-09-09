use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedFrameRequest,
    UiMountedFrameRetentionRejection, UiMountedHostMeasurementTransitionDenial,
    UiMountedHostMeasurementUnexpectedTransition, UiMountedIndeterminateFrame,
    UiMountedPresentationAdmissionRejection, UiMountedPresentationCompletionDenial,
    UiMountedPresentationInFlight, UiMountedRejectedFrame, UiMountedSupersededFrame,
    UiPresentationDeadline, WorthUi, WorthUiMountedFrameExecutionStop,
};

fn main() {
    run();
}

pub fn run() {
    let app = WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .map(
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_headless,
        )
        .expect("empty application preparation should succeed");
    let mut session = app.launch().expect("empty application should launch");
    let outcome = match session.execute_mounted_frame(
        UiMountedFrameRequest::all_bound_surfaces(),
        UiPresentationDeadline::at_tick(1),
        0,
        |_| {},
    ) {
        Ok(outcome) => outcome,
        Err(stop) => {
            observe_stop(&stop);
            return;
        }
    };

    match outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => {
            observe_publication(&receipt);
        }
        UiMountedFrameOutcome::RejectedBeforeEffects(rejection) => {
            observe_rejection(&rejection);
        }
        UiMountedFrameOutcome::InFlight(in_flight) => {
            observe_in_flight(&in_flight);
        }
        UiMountedFrameOutcome::PresentationIndeterminate(indeterminate) => {
            observe_indeterminate(&indeterminate);
        }
        UiMountedFrameOutcome::RetentionDenied(rejection) => {
            observe_retention_denial(&rejection);
        }
        UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            observe_admission_denial(&rejection);
        }
        UiMountedFrameOutcome::CompletionDenied(denial) => observe_completion_denial(&denial),
        UiMountedFrameOutcome::Superseded(superseded) => observe_superseded(&superseded),
    }
}

fn observe_stop(stop: &WorthUiMountedFrameExecutionStop<'_>) {
    match stop {
        WorthUiMountedFrameExecutionStop::PublicationLease(_) => {}
        WorthUiMountedFrameExecutionStop::HostMeasurement(_) => {}
        WorthUiMountedFrameExecutionStop::HostMeasurementTransition(denial) => {
            observe_host_measurement_transition(denial)
        }
        WorthUiMountedFrameExecutionStop::FrameworkTransition(transition) => {
            let _ = transition.generation_identity();
        }
        WorthUiMountedFrameExecutionStop::OccurrenceGeometry(_) => {}
        WorthUiMountedFrameExecutionStop::Preparation(_) => {}
    }
}

fn observe_publication(receipt: &UiMountedFramePublicationReceipt) {
    let _ = receipt.cost_report();
}

fn observe_rejection(rejection: &UiMountedRejectedFrame) {
    let _ = rejection.cost_report();
}

fn observe_in_flight(in_flight: &UiMountedPresentationInFlight) {
    let _ = in_flight.cost_report();
}

fn observe_indeterminate(indeterminate: &UiMountedIndeterminateFrame) {
    let _ = indeterminate.cost_report();
}

fn observe_retention_denial(rejection: &UiMountedFrameRetentionRejection) {
    let _ = rejection.frame().cost_report();
}

fn observe_admission_denial(rejection: &UiMountedPresentationAdmissionRejection) {
    let _ = rejection.frame().cost_report();
}

fn observe_completion_denial(_denial: &UiMountedPresentationCompletionDenial) {}

fn observe_superseded(superseded: &UiMountedSupersededFrame) {
    let _ = superseded.cost_report();
}

fn observe_host_measurement_transition(denial: &UiMountedHostMeasurementTransitionDenial) {
    use UiMountedHostMeasurementTransitionDenial as Denial;
    use UiMountedHostMeasurementUnexpectedTransition as Unexpected;

    match denial {
        Denial::AllocationReplanDenied(_)
        | Denial::ViewportResizeDenied(_)
        | Denial::AllocationReplanSelectionDenied(_)
        | Denial::AllocationFrameResolutionDenied(_)
        | Denial::AllocationInvalidationNarrowingDenied(_)
        | Denial::FrameworkTransitionPlanningDenied(_)
        | Denial::FrameworkTransitionExecutionDenied(_)
        | Denial::DispatcherDenied { .. } => {}
        Denial::UnexpectedSuccessfulTransition(unexpected) => match unexpected {
            Unexpected::ReadyToExecute
            | Unexpected::ResizePreviewPublished
            | Unexpected::DurableResizeCommitted
            | Unexpected::DragResizePreviewPending => {}
        },
    }
}
