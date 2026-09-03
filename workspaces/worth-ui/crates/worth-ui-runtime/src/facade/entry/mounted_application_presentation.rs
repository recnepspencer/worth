use super::{
    mounted_publication::finish_mounted_transition, WorthUiActiveApplicationSession,
    WorthUiMountedFrameExecutionStop, WorthUiMountedFrameFrameworkTransitionStop,
};
use crate::mounting::UiMountedFrameOutcome;
use worth_ui_host_contract::UiPresentationDeadline;

type PendingHostMeasurements = (
    crate::facade::WorthUiHostMeasurementCapability,
    Vec<crate::facade::WorthUiHostMeasurementSessionInput>,
);

#[derive(Debug)]
pub enum UiMountedHostMeasurementTransitionDenial {
    AllocationReplanDenied(Box<crate::runtime::UiAllocationReplanTransactionCommitDenial>),
    ViewportResizeDenied(Box<crate::runtime::UiViewportResizeDenial>),
    AllocationReplanSelectionDenied(Box<crate::graph::UiReplanLocalityDenial>),
    AllocationFrameResolutionDenied(Box<crate::runtime::UiAllocationFrameRejection>),
    AllocationInvalidationNarrowingDenied(
        Box<crate::runtime::UiAllocationInvalidationNarrowingRejection>,
    ),
    FrameworkTransitionPlanningDenied(Box<crate::runtime::UiFrameworkTransitionPlanningDenial>),
    FrameworkTransitionExecutionDenied(Box<crate::runtime::UiFrameworkTransitionExecutionDenial>),
    DispatcherDenied {
        denial: Box<crate::runtime::UiAllocationFrameDispatchDenial>,
        counters: crate::runtime::UiAllocationFrameDispatcherCounters,
    },
    UnexpectedSuccessfulTransition(UiMountedHostMeasurementUnexpectedTransition),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedHostMeasurementUnexpectedTransition {
    ReadyToExecute,
    ResizePreviewPublished,
    DurableResizeCommitted,
    DragResizePreviewPending,
}

pub(crate) enum UiMountedHostMeasurementSettlementStop {
    PublicationLease(crate::mounting::UiMountedPublicationLeaseDenial),
    Evidence(Box<crate::facade::host::UiHostMeasurementEvidenceDenial>),
    Transition(Box<UiMountedHostMeasurementTransitionDenial>),
}

impl<'session> From<UiMountedHostMeasurementSettlementStop>
    for WorthUiMountedFrameExecutionStop<'session>
{
    fn from(stop: UiMountedHostMeasurementSettlementStop) -> Self {
        match stop {
            UiMountedHostMeasurementSettlementStop::PublicationLease(denial) => {
                Self::PublicationLease(denial)
            }
            UiMountedHostMeasurementSettlementStop::Evidence(denial) => {
                Self::HostMeasurement(denial)
            }
            UiMountedHostMeasurementSettlementStop::Transition(denial) => {
                Self::HostMeasurementTransition(denial)
            }
        }
    }
}

impl WorthUiActiveApplicationSession {
    pub(crate) fn settle_mounted_host_measurements(
        &mut self,
        host_measurements: Option<PendingHostMeasurements>,
    ) -> Result<(), UiMountedHostMeasurementSettlementStop> {
        let Some((capability, inputs)) = host_measurements else {
            return Ok(());
        };
        let mut evidence_denial = None;
        let completion = self
            .execute_framework_turn(|turn| {
                turn.host_measurement(|source| {
                    for input in inputs {
                        if let Err(observed) =
                            source.collect_and_submit_capability(&capability, input)
                        {
                            evidence_denial.get_or_insert(observed);
                        }
                    }
                });
            })
            .map_err(UiMountedHostMeasurementSettlementStop::PublicationLease)?;
        if let Some(denial) = evidence_denial {
            drop(completion);
            return Err(UiMountedHostMeasurementSettlementStop::Evidence(Box::new(
                denial,
            )));
        }
        require_committed_host_measurement_transition(completion)
            .map_err(|denial| UiMountedHostMeasurementSettlementStop::Transition(Box::new(denial)))
    }

    pub(crate) fn execute_mounted_frame_with_application_presentation(
        &mut self,
        deadline: UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, WorthUiMountedFrameExecutionStop<'_>> {
        let request = self.mounted_frame_request();
        let projection = self
            .presentation
            .project()
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        let completion = self
            .execute_framework_turn(|_| {})
            .map_err(WorthUiMountedFrameExecutionStop::PublicationLease)?;
        let mut execution = completion.into_execution().map_err(|completion| {
            WorthUiMountedFrameExecutionStop::FrameworkTransition(
                WorthUiMountedFrameFrameworkTransitionStop { completion },
            )
        })?;
        let frame = execution
            .prepare_mounted_frame_with_content_internal(
                request,
                projection.content(),
                projection.theme_values(),
            )
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        execution.presentation.commit(&projection);
        let transition = execution.mounted.present_prepared_frame(
            execution.host_session,
            frame,
            Some(execution.appearance_inspection),
            deadline,
            now,
        );
        Ok(finish_mounted_transition(
            execution.mounted,
            execution.focus,
            execution.portal,
            execution.interaction,
            execution.host_session,
            execution.application_session_identity,
            &execution.generation_identity,
            execution.host_exchange,
            transition,
        ))
    }

    pub(crate) fn execute_mounted_rebound_frame_with_application_presentation(
        &mut self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        deadline: UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, WorthUiMountedFrameExecutionStop<'_>> {
        let request = self.mounted_frame_request();
        let projection = self
            .presentation
            .project()
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        let completion = self
            .execute_framework_turn(|_| {})
            .map_err(WorthUiMountedFrameExecutionStop::PublicationLease)?;
        let mut execution = completion.into_execution().map_err(|completion| {
            WorthUiMountedFrameExecutionStop::FrameworkTransition(
                WorthUiMountedFrameFrameworkTransitionStop { completion },
            )
        })?;
        let frame = execution
            .prepare_mounted_reconciliation_frame_with_content_internal(
                request,
                projection.content(),
                projection.theme_values(),
                replacements,
            )
            .map_err(|denial| WorthUiMountedFrameExecutionStop::Preparation(Box::new(denial)))?;
        execution.presentation.commit(&projection);
        let transition = execution.mounted.present_prepared_frame(
            execution.host_session,
            frame,
            Some(execution.appearance_inspection),
            deadline,
            now,
        );
        Ok(finish_mounted_transition(
            execution.mounted,
            execution.focus,
            execution.portal,
            execution.interaction,
            execution.host_session,
            execution.application_session_identity,
            &execution.generation_identity,
            execution.host_exchange,
            transition,
        ))
    }
}

fn require_committed_host_measurement_transition(
    completion: super::WorthUiActiveFrameworkTurnCompletion<'_>,
) -> Result<(), UiMountedHostMeasurementTransitionDenial> {
    use crate::runtime::WorthUiFrameworkTurnCompletion as Completion;
    use UiMountedHostMeasurementTransitionDenial as Denial;
    use UiMountedHostMeasurementUnexpectedTransition as Unexpected;

    match completion.into_completion() {
        Completion::AllocationInvalidationsNarrowed {
            transaction:
                crate::runtime::UiAllocationReplanTransactionOutcome::Committed(_)
                | crate::runtime::UiAllocationReplanTransactionOutcome::Replayed(_),
            ..
        }
        | Completion::ViewportResizeResolved { .. } => Ok(()),
        Completion::AllocationInvalidationsNarrowed {
            transaction: crate::runtime::UiAllocationReplanTransactionOutcome::Denied(denial),
            ..
        } => Err(Denial::AllocationReplanDenied(Box::new(denial))),
        Completion::ViewportResizeDenied { denial, .. } => {
            Err(Denial::ViewportResizeDenied(Box::new(denial)))
        }
        Completion::AllocationReplanSelectionDenied { denial } => {
            Err(Denial::AllocationReplanSelectionDenied(Box::new(denial)))
        }
        Completion::AllocationFrameResolutionDenied { rejection } => {
            Err(Denial::AllocationFrameResolutionDenied(Box::new(rejection)))
        }
        Completion::AllocationInvalidationNarrowingDenied { rejection } => Err(
            Denial::AllocationInvalidationNarrowingDenied(Box::new(rejection)),
        ),
        Completion::FrameworkTransitionPlanningDenied { denial } => {
            Err(Denial::FrameworkTransitionPlanningDenied(Box::new(denial)))
        }
        Completion::FrameworkTransitionExecutionDenied { denial } => {
            Err(Denial::FrameworkTransitionExecutionDenied(Box::new(denial)))
        }
        Completion::Denied { denial, counters } => Err(Denial::DispatcherDenied {
            denial: Box::new(denial),
            counters,
        }),
        Completion::ReadyToExecute { .. } => Err(Denial::UnexpectedSuccessfulTransition(
            Unexpected::ReadyToExecute,
        )),
        Completion::ResizePreviewPublished { .. } => Err(Denial::UnexpectedSuccessfulTransition(
            Unexpected::ResizePreviewPublished,
        )),
        Completion::DurableResizeCommitted { .. } => Err(Denial::UnexpectedSuccessfulTransition(
            Unexpected::DurableResizeCommitted,
        )),
        Completion::DragResizePreviewPending { .. } => Err(Denial::UnexpectedSuccessfulTransition(
            Unexpected::DragResizePreviewPending,
        )),
    }
}
