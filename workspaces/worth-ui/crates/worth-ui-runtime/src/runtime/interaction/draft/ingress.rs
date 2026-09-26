use worth_ui_host_contract::{
    UiHostImeCompositionPhase, UiHostKey, UiHostKeyTransition, UiHostObservationCanonicalCore,
    UiHostObservationPayload, UiHostObservationReport, UiHostObservationSequence,
    UiHostObservationTimeBasis,
};

use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use super::model::{
    UiActiveLocalRecipient, UiDraftProcessingOutcome, UiDraftRuntimeState, UiRecipientContext,
    UiValidatedActiveRecipient,
};
use super::{UiDraftMutationKind, UiLocalInputStopReason};
use crate::runtime::interaction::{
    UiActivateInteraction, UiKeyboardActivationEvidence, UiKeyboardSemanticInput,
    UiSemanticInteraction, UiSubmitInteraction,
};

#[derive(Clone, Copy)]
struct UiDraftReportContext<'world> {
    core: UiHostObservationCanonicalCore,
    sequence: UiHostObservationSequence,
    time_basis: UiHostObservationTimeBasis,
    mounted: &'world crate::mounting::WorthUiMountedSessionState,
    generation: &'world WorthUiActiveApplicationGenerationIdentity,
}

impl UiDraftRuntimeState {
    pub(crate) fn process_report(
        &mut self,
        core: UiHostObservationCanonicalCore,
        report: &UiHostObservationReport,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &WorthUiActiveApplicationGenerationIdentity,
    ) -> Vec<UiDraftProcessingOutcome> {
        let context = UiDraftReportContext {
            core,
            sequence: report.sequence(),
            time_basis: report.time_basis(),
            mounted,
            generation,
        };
        if report_requires_recipient_affinity(report.payload()) {
            if let Some(rejection) = self.reject_invalid_affinity(context, report) {
                return rejection;
            }
        }
        match report.payload() {
            UiHostObservationPayload::Keyboard {
                logical_key,
                modifiers,
                transition,
                ..
            } => self.process_key(context, *logical_key, *modifiers, *transition),
            UiHostObservationPayload::TextInput { revision, text } => self.process_committed_text(
                context,
                *revision,
                text,
                UiDraftMutationKind::CommittedText,
            ),
            UiHostObservationPayload::ImeComposition { revision, phase } => {
                self.process_ime(context, *revision, phase)
            }
            UiHostObservationPayload::WindowFocus { focused: false, .. } => self
                .cancel_all(UiLocalInputStopReason::FocusLost)
                .into_iter()
                .map(UiDraftProcessingOutcome::Stopped)
                .collect(),
            UiHostObservationPayload::Viewport { .. }
            | UiHostObservationPayload::DeviceScale { .. }
            | UiHostObservationPayload::PointerMotion { .. }
            | UiHostObservationPayload::PointerButton { .. }
            | UiHostObservationPayload::WindowFocus { .. }
            | UiHostObservationPayload::ScrollDelta { .. }
            | UiHostObservationPayload::Clock { .. }
            | UiHostObservationPayload::Tick { .. } => Vec::new(),
        }
    }

    fn process_key(
        &mut self,
        context: UiDraftReportContext<'_>,
        key: UiHostKey,
        modifiers: worth_ui_host_contract::UiHostKeyboardModifiers,
        transition: UiHostKeyTransition,
    ) -> Vec<UiDraftProcessingOutcome> {
        let UiHostKeyTransition::Pressed { repeat: false } = transition else {
            return Vec::new();
        };
        let Some(active) = self.validate_active(context) else {
            return self.missing_or_invalid_active(context);
        };
        match active {
            UiValidatedActiveRecipient::Activation(target) if activation_key(key) => {
                vec![UiDraftProcessingOutcome::Semantic(
                    UiSemanticInteraction::Activate(UiActivateInteraction::from_keyboard(
                        UiKeyboardActivationEvidence::new(UiKeyboardSemanticInput {
                            target,
                            presentation: context.core.presentation(),
                            generation: context.generation.clone(),
                            sequence: context.sequence,
                            time_basis: context.time_basis,
                            key,
                            modifiers,
                        }),
                    )),
                )]
            }
            UiValidatedActiveRecipient::Submit(target) if key == UiHostKey::Enter => {
                vec![UiDraftProcessingOutcome::Semantic(
                    UiSemanticInteraction::Submit(UiSubmitInteraction::seal(
                        UiKeyboardSemanticInput {
                            target,
                            presentation: context.core.presentation(),
                            generation: context.generation.clone(),
                            sequence: context.sequence,
                            time_basis: context.time_basis,
                            key,
                            modifiers,
                        },
                    )),
                )]
            }
            UiValidatedActiveRecipient::Activation(_) | UiValidatedActiveRecipient::Submit(_)
                if key == UiHostKey::Escape =>
            {
                vec![UiDraftProcessingOutcome::DismissRequested(
                    crate::runtime::interaction::UiDismissInteraction::escape(
                        context.core.presentation(),
                        context.sequence,
                        context.time_basis,
                    ),
                )]
            }
            UiValidatedActiveRecipient::Draft(session) if key == UiHostKey::Escape => self
                .cancel_session(session, UiLocalInputStopReason::ExplicitCancel)
                .into_iter()
                .map(UiDraftProcessingOutcome::Stopped)
                .collect(),
            UiValidatedActiveRecipient::Draft(session) if key == UiHostKey::Backspace => self
                .backspace(session, context.sequence)
                .into_iter()
                .collect(),
            UiValidatedActiveRecipient::Draft(session) if key == UiHostKey::Enter => {
                vec![self.commit(
                    session,
                    context.core.presentation(),
                    context.sequence,
                    context.time_basis,
                )]
            }
            UiValidatedActiveRecipient::Activation(_)
            | UiValidatedActiveRecipient::Draft(_)
            | UiValidatedActiveRecipient::Submit(_) => Vec::new(),
        }
    }

    fn process_committed_text(
        &mut self,
        context: UiDraftReportContext<'_>,
        revision: u64,
        text: &str,
        kind: UiDraftMutationKind,
    ) -> Vec<UiDraftProcessingOutcome> {
        let session = match self.validate_active(context) {
            Some(UiValidatedActiveRecipient::Draft(session)) => session,
            Some(active) => {
                return vec![UiDraftProcessingOutcome::Stopped(self.unsettled_stop(
                    context.core,
                    UiLocalInputStopReason::RecipientFamilyMismatch {
                        required: super::UiLocalInputRecipientFamily::Draft,
                        active: active.family(),
                    },
                ))];
            }
            None => return self.missing_or_invalid_active(context),
        };
        vec![self.apply_committed_text(
            session,
            super::mutation::UiCommittedTextMutation {
                sequence: context.sequence,
                revision,
                text,
                kind,
            },
        )]
    }

    fn process_ime(
        &mut self,
        context: UiDraftReportContext<'_>,
        revision: u64,
        phase: &UiHostImeCompositionPhase,
    ) -> Vec<UiDraftProcessingOutcome> {
        let session = match self.validate_active(context) {
            Some(UiValidatedActiveRecipient::Draft(session)) => session,
            Some(active) => {
                return vec![UiDraftProcessingOutcome::Stopped(self.unsettled_stop(
                    context.core,
                    UiLocalInputStopReason::RecipientFamilyMismatch {
                        required: super::UiLocalInputRecipientFamily::Draft,
                        active: active.family(),
                    },
                ))];
            }
            None => return self.missing_or_invalid_active(context),
        };
        vec![match phase {
            UiHostImeCompositionPhase::Preedit(preedit) => {
                self.apply_preedit(session, context.sequence, revision, preedit.clone())
            }
            UiHostImeCompositionPhase::Commit(text) => self.apply_committed_text(
                session,
                super::mutation::UiCommittedTextMutation {
                    sequence: context.sequence,
                    revision,
                    text,
                    kind: UiDraftMutationKind::PreeditCommit,
                },
            ),
            UiHostImeCompositionPhase::Cancel => {
                self.cancel_preedit(session, context.sequence, revision)
            }
        }]
    }

    fn validate_active(
        &self,
        report: UiDraftReportContext<'_>,
    ) -> Option<UiValidatedActiveRecipient> {
        let context = self.active_context()?;
        if context.target.binding() != report.core.binding()
            || &context.generation != report.generation
            || crate::runtime::interaction::targeting::require_current_target(
                report.mounted,
                context.target,
            )
            .is_err()
        {
            return None;
        }
        Some(match self.active.as_ref()? {
            UiActiveLocalRecipient::Activation(_) => {
                UiValidatedActiveRecipient::Activation(context.target)
            }
            UiActiveLocalRecipient::Draft(session) => UiValidatedActiveRecipient::Draft(*session),
            UiActiveLocalRecipient::Submit(_) => UiValidatedActiveRecipient::Submit(context.target),
        })
    }

    fn reject_invalid_affinity(
        &mut self,
        context: UiDraftReportContext<'_>,
        report: &UiHostObservationReport,
    ) -> Option<Vec<UiDraftProcessingOutcome>> {
        let Some(lease) = self.active_affinity else {
            return Some(vec![UiDraftProcessingOutcome::Stopped(
                self.unsettled_stop(context.core, UiLocalInputStopReason::NoLocalRecipient),
            )]);
        };
        if report.input_affinity().is_none() {
            return Some(vec![UiDraftProcessingOutcome::Stopped(
                self.unsettled_stop(
                    context.core,
                    UiLocalInputStopReason::MissingInputRecipientAffinity,
                ),
            )]);
        }
        if let Some((expected, observed)) =
            lease.reported_text_profile_mismatch(report, context.core.presentation())
        {
            return Some(vec![UiDraftProcessingOutcome::Stopped(
                self.unsettled_stop(
                    context.core,
                    UiLocalInputStopReason::TextProfileGenerationChanged { expected, observed },
                ),
            )]);
        }
        if !lease.admits_report(report, context.core.presentation()) {
            return Some(vec![UiDraftProcessingOutcome::Stopped(
                self.unsettled_stop(
                    context.core,
                    UiLocalInputStopReason::InputRecipientAffinityChanged,
                ),
            )]);
        }
        if !payload_requires_text_profile(report.payload()) {
            return None;
        }
        let binding = lease.binding();
        let expected = binding.text_profile()?;
        let target = self
            .active_context()
            .expect("live input affinity has an active recipient")
            .target;
        let observed = context.mounted.input_text_profile(target);
        if observed == Some(expected) {
            return None;
        }
        Some(
            self.cancel_active(UiLocalInputStopReason::TextProfileGenerationChanged {
                expected,
                observed,
            })
            .into_iter()
            .map(UiDraftProcessingOutcome::Stopped)
            .collect(),
        )
    }

    fn missing_or_invalid_active(
        &mut self,
        report: UiDraftReportContext<'_>,
    ) -> Vec<UiDraftProcessingOutcome> {
        let Some(context) = self.active_context() else {
            return vec![UiDraftProcessingOutcome::Stopped(self.unsettled_stop(
                report.core,
                UiLocalInputStopReason::NoLocalRecipient,
            ))];
        };
        if context.target.binding() != report.core.binding() {
            return vec![UiDraftProcessingOutcome::Stopped(self.unsettled_stop(
                report.core,
                UiLocalInputStopReason::ForeignBinding {
                    expected: context.target.binding(),
                    observed: report.core.binding(),
                },
            ))];
        }
        let reason = if &context.generation != report.generation {
            UiLocalInputStopReason::ApplicationGenerationChanged
        } else {
            let denial = crate::runtime::interaction::targeting::require_current_target(
                report.mounted,
                context.target,
            )
            .expect_err("the active target was not current");
            UiLocalInputStopReason::TargetNoLongerCurrent(denial)
        };
        self.cancel_active(reason)
            .into_iter()
            .map(UiDraftProcessingOutcome::Stopped)
            .collect()
    }

    pub(super) fn active_context(&self) -> Option<UiRecipientContext> {
        match self.active.as_ref()? {
            UiActiveLocalRecipient::Activation(context)
            | UiActiveLocalRecipient::Submit(context) => Some(context.clone()),
            UiActiveLocalRecipient::Draft(session) => {
                let draft = self.sessions.get(session)?;
                Some(UiRecipientContext {
                    target: draft.target,
                    generation: draft.generation.clone(),
                })
            }
        }
    }
}

fn activation_key(key: UiHostKey) -> bool {
    matches!(key, UiHostKey::Enter | UiHostKey::Space)
}

fn report_requires_recipient_affinity(payload: &UiHostObservationPayload) -> bool {
    matches!(
        payload,
        UiHostObservationPayload::Keyboard { .. }
            | UiHostObservationPayload::TextInput { .. }
            | UiHostObservationPayload::ImeComposition { .. }
    )
}

fn payload_requires_text_profile(payload: &UiHostObservationPayload) -> bool {
    matches!(
        payload,
        UiHostObservationPayload::TextInput { .. }
            | UiHostObservationPayload::ImeComposition { .. }
    )
}
