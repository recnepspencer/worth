use super::WorthUiActiveApplicationSession;
use std::collections::BTreeMap;
use worth_ui_host_contract::UiMountedPresentationAttemptIdentity;

pub(super) struct UiPreparedMountedOwnerReceiptSuccession {
    operability: crate::runtime::intent::UiPreparedIntentOperabilityReceiptSuccession,
    validation: crate::runtime::intent::UiPreparedValidationAppearanceReceiptSuccession,
}

impl UiPreparedMountedOwnerReceiptSuccession {
    fn prepare(
        mounted: &crate::mounting::WorthUiMountedSessionState,
        intent_admission: &crate::runtime::intent::UiIntentAdmissionState,
        intent_application_facts: &crate::runtime::intent::UiIntentApplicationFactState,
        successor: &crate::mounting::UiMountedNodeReceiptBasis,
    ) -> Self {
        Self {
            operability: intent_admission
                .prepare_operability_receipt_succession(mounted, successor),
            validation: intent_application_facts
                .prepare_validation_receipt_succession(mounted, successor),
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "mounted settlement refreshes each independent appearance owner"
    )]
    fn commit(
        self,
        focus: Option<&crate::runtime::focus::UiFocusRuntimeState>,
        selection: Option<&crate::runtime::selection::UiSelectionRuntimeState>,
        intent_admission: &mut crate::runtime::intent::UiIntentAdmissionState,
        intent_application_facts: &mut crate::runtime::intent::UiIntentApplicationFactState,
        interaction: &crate::runtime::interaction::UiInteractionRuntimeState,
        appearance_owner_snapshot: &mut Option<
            crate::runtime::appearance::UiAppearanceOwnerSnapshot,
        >,
    ) {
        assert!(intent_admission.admits_operability_receipt_succession(&self.operability));
        assert!(intent_application_facts.admits_validation_receipt_succession(&self.validation));
        intent_admission.commit_operability_receipt_succession(self.operability);
        intent_application_facts.commit_validation_receipt_succession(self.validation);
        *appearance_owner_snapshot = appearance_owner_snapshot.as_ref().map(|owners| {
            owners
                .refresh_receipt_sources(
                    focus,
                    selection,
                    intent_admission,
                    intent_application_facts,
                    interaction,
                )
                .expect("accepted mounted receipt succession retains each demanded owner")
        });
    }

    fn is_current(
        &self,
        intent_admission: &crate::runtime::intent::UiIntentAdmissionState,
        intent_application_facts: &crate::runtime::intent::UiIntentApplicationFactState,
    ) -> bool {
        intent_admission.admits_operability_receipt_succession(&self.operability)
            && intent_application_facts.admits_validation_receipt_succession(&self.validation)
    }
}

#[derive(Default)]
pub(super) struct UiMountedOwnerReceiptSuccessionCoordinator {
    pending:
        BTreeMap<UiMountedPresentationAttemptIdentity, UiPreparedMountedOwnerReceiptSuccession>,
}

impl WorthUiActiveApplicationSession {
    pub(super) fn prepare_mounted_owner_receipt_succession(
        &self,
        frame: &crate::mounting::UiPreparedMountedFrame,
    ) -> UiPreparedMountedOwnerReceiptSuccession {
        UiPreparedMountedOwnerReceiptSuccession::prepare(
            &self.mounted,
            &self.intent_admission,
            &self.intent_application_facts,
            frame.presented_receipt_basis(),
        )
    }

    pub(super) fn commit_mounted_owner_receipt_succession(
        &mut self,
        prepared: UiPreparedMountedOwnerReceiptSuccession,
    ) {
        prepared.commit(
            self.focus.as_ref(),
            self.selection.as_ref(),
            &mut self.intent_admission,
            &mut self.intent_application_facts,
            &self.interaction,
            &mut self.appearance_owner_snapshot,
        );
    }

    pub(super) fn refresh_appearance_owner_receipt_sources(&mut self) {
        self.appearance_owner_snapshot = self.current_appearance_owner_receipt_sources();
    }

    pub(super) fn refresh_motion_appearance_owner_receipt_sources(&mut self) {
        let refreshed = self.current_appearance_owner_receipt_sources();
        self.queue_closed_owner_invalidation(refreshed.as_ref());
        self.appearance_owner_snapshot = refreshed;
    }

    fn current_appearance_owner_receipt_sources(
        &self,
    ) -> Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.appearance_owner_snapshot.as_ref().map(|owners| {
            owners
                .refresh_receipt_sources(
                    self.focus.as_ref(),
                    self.selection.as_ref(),
                    &self.intent_admission,
                    &self.intent_application_facts,
                    &self.interaction,
                )
                .expect("accepted mounted receipt succession retains each demanded owner")
        })
    }

    pub(super) fn settle_new_mounted_owner_receipt_succession(
        &mut self,
        prepared: UiPreparedMountedOwnerReceiptSuccession,
        outcome: &crate::mounting::UiMountedFrameOutcome,
    ) {
        match outcome {
            crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
                self.commit_mounted_owner_receipt_succession(prepared);
            }
            crate::mounting::UiMountedFrameOutcome::InFlight(pending) => {
                self.mounted_owner_receipt_successions
                    .retain(pending.attempt(), prepared);
            }
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_)
            | crate::mounting::UiMountedFrameOutcome::Superseded(_)
            | crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {}
        }
    }

    pub(super) fn settle_pending_mounted_owner_receipt_succession(
        &mut self,
        outcome: &crate::mounting::UiMountedFrameOutcome,
    ) {
        let (attempt, accepted, terminal) = match outcome {
            crate::mounting::UiMountedFrameOutcome::Published(receipt)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(receipt)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(receipt) => {
                (Some(receipt.attempt()), true, true)
            }
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
                (Some(rejected.attempt()), false, true)
            }
            crate::mounting::UiMountedFrameOutcome::Superseded(superseded) => {
                (Some(superseded.attempt()), false, true)
            }
            crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejected) => {
                (rejected.attempt(), false, true)
            }
            crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(indeterminate) => {
                (Some(indeterminate.report().attempt()), false, true)
            }
            crate::mounting::UiMountedFrameOutcome::InFlight(_)
            | crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => (None, false, false),
        };
        if !terminal {
            return;
        }
        let Some(prepared) =
            attempt.and_then(|attempt| self.mounted_owner_receipt_successions.take(attempt))
        else {
            return;
        };
        if accepted {
            self.commit_mounted_owner_receipt_succession(prepared);
        }
    }

    pub(super) fn pending_mounted_owner_receipt_succession_is_current(
        &self,
        attempt: UiMountedPresentationAttemptIdentity,
    ) -> bool {
        self.mounted_owner_receipt_successions
            .get(attempt)
            .is_none_or(|prepared| {
                prepared.is_current(&self.intent_admission, &self.intent_application_facts)
            })
    }
}

impl UiMountedOwnerReceiptSuccessionCoordinator {
    pub(super) fn retain(
        &mut self,
        attempt: UiMountedPresentationAttemptIdentity,
        prepared: UiPreparedMountedOwnerReceiptSuccession,
    ) {
        assert!(self.pending.insert(attempt, prepared).is_none());
    }

    pub(super) fn take(
        &mut self,
        attempt: UiMountedPresentationAttemptIdentity,
    ) -> Option<UiPreparedMountedOwnerReceiptSuccession> {
        self.pending.remove(&attempt)
    }

    fn get(
        &self,
        attempt: UiMountedPresentationAttemptIdentity,
    ) -> Option<&UiPreparedMountedOwnerReceiptSuccession> {
        self.pending.get(&attempt)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

impl super::WorthUiActiveFrameworkTurnExecution<'_> {
    pub(super) fn prepare_mounted_owner_receipts(
        &self,
        frame: &crate::mounting::UiPreparedMountedFrame,
    ) -> UiPreparedMountedOwnerReceiptSuccession {
        UiPreparedMountedOwnerReceiptSuccession::prepare(
            self.mounted,
            self.intent_admission,
            self.intent_application_facts,
            frame.presented_receipt_basis(),
        )
    }

    pub(super) fn prepare_mounted_owner_receipts_for_assembled(
        &self,
        frame: &crate::mounting::UiAssembledMountedFrame,
    ) -> UiPreparedMountedOwnerReceiptSuccession {
        UiPreparedMountedOwnerReceiptSuccession::prepare(
            self.mounted,
            self.intent_admission,
            self.intent_application_facts,
            frame.presented_receipt_basis(),
        )
    }

    pub(super) fn settle_new_mounted_owner_receipts(
        &mut self,
        prepared: UiPreparedMountedOwnerReceiptSuccession,
        outcome: &crate::mounting::UiMountedFrameOutcome,
    ) {
        match outcome {
            crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => prepared.commit(
                self.focus.as_deref(),
                self.selection.as_deref(),
                self.intent_admission,
                self.intent_application_facts,
                self.interaction,
                self.appearance_owner_snapshot,
            ),
            crate::mounting::UiMountedFrameOutcome::InFlight(pending) => self
                .mounted_owner_receipt_successions
                .retain(pending.attempt(), prepared),
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_)
            | crate::mounting::UiMountedFrameOutcome::Superseded(_)
            | crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {}
        }
    }
}
