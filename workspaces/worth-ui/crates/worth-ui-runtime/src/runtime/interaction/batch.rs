use worth_ui_host_contract::UiHostObservationCanonicalCore;

use super::{
    UiInteractionLifecycleSettlementReceipt, UiInteractionStateSnapshot, UiInteractionTransition,
};

#[derive(Debug)]
pub struct UiInteractionBatchReceipt {
    pub(super) core: UiHostObservationCanonicalCore,
    pub(super) frame_relation: crate::facade::observation_report::UiHostObservationFrameRelation,
    pub(super) disposition: crate::facade::observation_report::UiHostObservationBatchDisposition,
    pub(super) transitions: Box<[UiInteractionTransition]>,
    pub(super) ignored_reports: usize,
    pub(super) state: UiInteractionStateSnapshot,
    pub(super) scroll_observations: Box<[crate::runtime::scroll::UiHostScrollObservationOutcome]>,
    pub(super) command_routes: Box<[crate::runtime::UiCommandRoutingOutcome]>,
    #[allow(
        dead_code,
        reason = "Gate 0 exposes owner-issued transitions before Gate 1 live resolver threading"
    )]
    pub(super) pointer_presence_transitions:
        Box<[super::pointer_presence::UiPointerPresenceTargetTransition]>,
    pub(super) pointer_presence_denials:
        Box<[super::pointer_presence::UiPointerPresenceAdmissionDenial]>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiInteractionObservationDenial {
    pub(super) denial: crate::facade::observation_report::UiHostObservationReportDenial,
    pub(super) settlement: UiInteractionLifecycleSettlementReceipt,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiQuarantinedHostInteractionBatch {
    pub(super) quarantine: crate::facade::observation_report::UiQuarantinedHostObservationBatch,
    pub(super) settlement: UiInteractionLifecycleSettlementReceipt,
}

#[derive(Debug)]
pub enum UiHostInteractionIngressOutcome {
    Applied(UiInteractionBatchReceipt),
    Duplicate(crate::facade::observation_report::UiDuplicateHostObservationBatch),
    Quarantined(UiQuarantinedHostInteractionBatch),
    Denied(UiInteractionObservationDenial),
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct UiInteractionShutdownReport {
    pub(super) settlement: Option<UiInteractionLifecycleSettlementReceipt>,
}

impl UiInteractionBatchReceipt {
    pub(crate) fn retain_scroll_observations(
        &mut self,
        observations: Vec<crate::runtime::scroll::UiHostScrollObservationOutcome>,
    ) {
        self.scroll_observations = observations.into_boxed_slice();
    }

    pub(crate) fn scroll_observations(
        &self,
    ) -> &[crate::runtime::scroll::UiHostScrollObservationOutcome] {
        &self.scroll_observations
    }

    pub(crate) fn retain_command_routes(
        &mut self,
        routes: Vec<crate::runtime::UiCommandRoutingOutcome>,
    ) {
        self.command_routes = routes.into_boxed_slice();
    }

    pub fn command_routes(&self) -> &[crate::runtime::UiCommandRoutingOutcome] {
        &self.command_routes
    }

    pub(crate) fn into_routing_parts(
        self,
    ) -> (
        Box<[UiInteractionTransition]>,
        Box<[crate::runtime::UiCommandRoutingOutcome]>,
    ) {
        (self.transitions, self.command_routes)
    }

    pub fn pointer_presence_transitions(
        &self,
    ) -> &[super::pointer_presence::UiPointerPresenceTargetTransition] {
        &self.pointer_presence_transitions
    }

    pub fn pointer_presence_denials(
        &self,
    ) -> &[super::pointer_presence::UiPointerPresenceAdmissionDenial] {
        &self.pointer_presence_denials
    }

    pub(crate) fn retain_service_dismissal(&mut self, dismissal: super::UiDismissInteraction) {
        let already_retained = self.transitions.iter().any(|transition| {
            matches!(
                transition,
                UiInteractionTransition::DismissRequested(retained)
                    if retained.sequence() == dismissal.sequence()
                        && retained.cause() == dismissal.cause()
            )
        });
        if already_retained {
            return;
        }
        let retained = std::mem::take(&mut self.transitions).into_vec();
        let mut transitions = Vec::with_capacity(retained.len().saturating_add(1));
        transitions.extend(retained);
        transitions.push(UiInteractionTransition::DismissRequested(dismissal));
        self.transitions = transitions.into_boxed_slice();
    }

    pub const fn canonical_core(&self) -> UiHostObservationCanonicalCore {
        self.core
    }

    pub const fn frame_relation(
        &self,
    ) -> crate::facade::observation_report::UiHostObservationFrameRelation {
        self.frame_relation
    }

    pub const fn disposition(
        &self,
    ) -> crate::facade::observation_report::UiHostObservationBatchDisposition {
        self.disposition
    }

    pub fn transitions(&self) -> &[UiInteractionTransition] {
        &self.transitions
    }

    pub fn into_transitions(self) -> Box<[UiInteractionTransition]> {
        self.transitions
    }

    pub const fn ignored_reports(&self) -> usize {
        self.ignored_reports
    }

    pub const fn state(&self) -> UiInteractionStateSnapshot {
        self.state
    }
}

impl UiInteractionObservationDenial {
    pub(crate) const fn new(
        denial: crate::facade::observation_report::UiHostObservationReportDenial,
        settlement: UiInteractionLifecycleSettlementReceipt,
    ) -> Self {
        Self { denial, settlement }
    }

    pub const fn denial(&self) -> crate::facade::observation_report::UiHostObservationReportDenial {
        self.denial
    }

    pub const fn settlement(&self) -> &UiInteractionLifecycleSettlementReceipt {
        &self.settlement
    }
}

impl UiQuarantinedHostInteractionBatch {
    pub(crate) const fn new(
        quarantine: crate::facade::observation_report::UiQuarantinedHostObservationBatch,
        settlement: UiInteractionLifecycleSettlementReceipt,
    ) -> Self {
        Self {
            quarantine,
            settlement,
        }
    }

    pub const fn quarantine(
        &self,
    ) -> crate::facade::observation_report::UiQuarantinedHostObservationBatch {
        self.quarantine
    }

    pub const fn settlement(&self) -> &UiInteractionLifecycleSettlementReceipt {
        &self.settlement
    }
}

impl UiInteractionShutdownReport {
    pub fn cancelled_gestures(&self) -> usize {
        self.settlement
            .as_ref()
            .map_or(0, UiInteractionLifecycleSettlementReceipt::settled_gestures)
    }

    pub fn cancelled_draft_sessions(&self) -> usize {
        self.settlement.as_ref().map_or(
            0,
            UiInteractionLifecycleSettlementReceipt::settled_draft_sessions,
        )
    }

    pub fn final_state(&self) -> Option<UiInteractionStateSnapshot> {
        self.settlement
            .as_ref()
            .map(UiInteractionLifecycleSettlementReceipt::final_state)
    }

    pub const fn settlement(&self) -> Option<&UiInteractionLifecycleSettlementReceipt> {
        self.settlement.as_ref()
    }
}
