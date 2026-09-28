use crate::graph::{
    UiGraphAxisParticipation, UiGraphParticipationAxis, UiGraphParticipationEvidenceHandle,
    UiGraphParticipationPosture, UiGraphParticipationReasonCode, UiGraphParticipationReasonSource,
    UiGraphParticipationStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiGraphParticipationSeed {
    posture: UiGraphParticipationPosture,
}

impl UiGraphParticipationSeed {
    pub(crate) const fn new(posture: UiGraphParticipationPosture) -> Self {
        Self { posture }
    }

    pub fn posture(self) -> UiGraphParticipationPosture {
        self.posture
    }

    pub fn axis(self, axis: UiGraphParticipationAxis) -> UiGraphAxisParticipation {
        self.posture.axis(axis)
    }

    pub(crate) const fn from_attachment_and_role(
        query_bound: bool,
        service_bound: bool,
        diagnostic_surface: bool,
    ) -> Self {
        Self::new(UiGraphParticipationPosture::new([
            axis(
                UiGraphParticipationStatus::Admitted,
                UiGraphParticipationReasonSource::GraphInstantiation,
                UiGraphParticipationReasonCode::InstantiatedNodeExists,
                UiGraphParticipationEvidenceHandle::InstantiationPlan,
            ),
            axis(
                UiGraphParticipationStatus::Deferred,
                UiGraphParticipationReasonSource::MountEligibility,
                UiGraphParticipationReasonCode::MountedAxisAwaitsRuntimeMutation,
                UiGraphParticipationEvidenceHandle::MountEligibilitySeed,
            ),
            deferred_axis(UiGraphParticipationAxis::Visible),
            deferred_axis(UiGraphParticipationAxis::Layout),
            deferred_axis(UiGraphParticipationAxis::HitTest),
            deferred_axis(UiGraphParticipationAxis::Focus),
            deferred_axis(UiGraphParticipationAxis::Accessibility),
            deferred_axis(UiGraphParticipationAxis::Paint),
            deferred_axis(UiGraphParticipationAxis::Input),
            if query_bound {
                axis(
                    UiGraphParticipationStatus::Admitted,
                    UiGraphParticipationReasonSource::AttachmentPosture,
                    UiGraphParticipationReasonCode::QueryBindingAttached,
                    UiGraphParticipationEvidenceHandle::QueryBindingAttachment,
                )
            } else {
                axis(
                    UiGraphParticipationStatus::Withheld,
                    UiGraphParticipationReasonSource::AttachmentPosture,
                    UiGraphParticipationReasonCode::QueryBindingAbsent,
                    UiGraphParticipationEvidenceHandle::QueryBindingAttachment,
                )
            },
            if service_bound {
                axis(
                    UiGraphParticipationStatus::Admitted,
                    UiGraphParticipationReasonSource::AttachmentPosture,
                    UiGraphParticipationReasonCode::ServiceUsageAttached,
                    UiGraphParticipationEvidenceHandle::ServiceUsageAttachment,
                )
            } else {
                axis(
                    UiGraphParticipationStatus::Withheld,
                    UiGraphParticipationReasonSource::AttachmentPosture,
                    UiGraphParticipationReasonCode::ServiceUsageAbsent,
                    UiGraphParticipationEvidenceHandle::ServiceUsageAttachment,
                )
            },
            if diagnostic_surface {
                axis(
                    UiGraphParticipationStatus::Admitted,
                    UiGraphParticipationReasonSource::ContainmentClaim,
                    UiGraphParticipationReasonCode::DiagnosticSurfaceOwned,
                    UiGraphParticipationEvidenceHandle::DiagnosticContainmentClaim,
                )
            } else {
                axis(
                    UiGraphParticipationStatus::Withheld,
                    UiGraphParticipationReasonSource::ContainmentClaim,
                    UiGraphParticipationReasonCode::DiagnosticSurfaceAbsent,
                    UiGraphParticipationEvidenceHandle::DiagnosticContainmentClaim,
                )
            },
        ]))
    }
}

const fn axis(
    status: UiGraphParticipationStatus,
    source: UiGraphParticipationReasonSource,
    reason: UiGraphParticipationReasonCode,
    evidence_handle: UiGraphParticipationEvidenceHandle,
) -> UiGraphAxisParticipation {
    UiGraphAxisParticipation::new(status, source, reason, evidence_handle)
}

const fn deferred_axis(participation_axis: UiGraphParticipationAxis) -> UiGraphAxisParticipation {
    let reason = match participation_axis {
        UiGraphParticipationAxis::Visible => {
            UiGraphParticipationReasonCode::VisibleAxisAwaitsRuntimeMutation
        }
        UiGraphParticipationAxis::Layout => {
            UiGraphParticipationReasonCode::LayoutAxisAwaitsRuntimeMutation
        }
        UiGraphParticipationAxis::HitTest => {
            UiGraphParticipationReasonCode::HitTestAxisAwaitsRuntimeMutation
        }
        UiGraphParticipationAxis::Focus => {
            UiGraphParticipationReasonCode::FocusAxisAwaitsRuntimeMutation
        }
        UiGraphParticipationAxis::Accessibility => {
            UiGraphParticipationReasonCode::AccessibilityAxisAwaitsRuntimeMutation
        }
        UiGraphParticipationAxis::Paint => {
            UiGraphParticipationReasonCode::PaintAxisAwaitsRuntimeMutation
        }
        UiGraphParticipationAxis::Input => {
            UiGraphParticipationReasonCode::InputAxisAwaitsRuntimeMutation
        }
        UiGraphParticipationAxis::Exists
        | UiGraphParticipationAxis::Mounted
        | UiGraphParticipationAxis::QueryBound
        | UiGraphParticipationAxis::ServiceBound
        | UiGraphParticipationAxis::Diagnostic => {
            UiGraphParticipationReasonCode::VisibleAxisAwaitsRuntimeMutation
        }
    };

    axis(
        UiGraphParticipationStatus::Deferred,
        UiGraphParticipationReasonSource::ReservedRuntimeMutation,
        reason,
        UiGraphParticipationEvidenceHandle::ReservedRuntimeMutationLane,
    )
}
