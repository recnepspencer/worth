use super::{UiIntentAdmissionState, UiIntentOperabilityStandingOwner};

pub(crate) struct UiPreparedIntentAdmissionRebind {
    predecessor: Option<crate::runtime::intent::UiIntentOperabilityStandingFactSnapshot>,
    successor: Option<UiIntentOperabilityStandingOwner>,
}

impl UiPreparedIntentAdmissionRebind {
    pub(crate) fn appearance_snapshot(
        &self,
    ) -> Option<crate::runtime::intent::UiIntentOperabilityStandingFactSnapshot> {
        self.successor.as_ref().map(|owner| owner.snapshot())
    }
}

impl UiIntentAdmissionState {
    pub(crate) fn prepare_application_rebind(
        &self,
        enabled: bool,
    ) -> UiPreparedIntentAdmissionRebind {
        UiPreparedIntentAdmissionRebind {
            predecessor: self.operability_standing_snapshot(),
            successor: enabled.then(|| {
                self.standing_owner
                    .as_ref()
                    .map_or_else(UiIntentOperabilityStandingOwner::default, |owner| {
                        owner.prepare_cleared()
                    })
            }),
        }
    }

    pub(crate) fn admits_application_rebind(
        &self,
        prepared: &UiPreparedIntentAdmissionRebind,
    ) -> bool {
        self.operability_standing_snapshot() == prepared.predecessor
    }

    pub(crate) fn commit_application_rebind(
        &mut self,
        execution: &mut crate::runtime::intent_execution::UiIntentExecutionState,
        prepared: UiPreparedIntentAdmissionRebind,
    ) -> usize {
        assert!(
            self.admits_application_rebind(&prepared),
            "accepted replacement retains prepared operability owner"
        );
        let cancelled = execution.cancel_all(
            crate::runtime::intent::UiIntentAdmissionCancellationReason::ApplicationRebound,
        );
        self.standing_owner = prepared.successor;
        self.record_lifecycle_cancellation(cancelled)
    }
}
