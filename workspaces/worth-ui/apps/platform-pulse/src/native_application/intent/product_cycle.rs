use super::super::PlatformPulseApplicationRuntime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::native_application) enum PlatformPulseIntentExecutionProgress {
    Idle,
    Progressed,
    AwaitingExternal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::native_application) enum PlatformPulseIntentProductCycleOutcome {
    Quiescent { rounds: usize },
    AwaitingExternal { rounds: usize },
    Interrupted { rounds: usize },
}

impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn drain_intent_product_cycle(
        &mut self,
    ) -> PlatformPulseIntentProductCycleOutcome {
        let mut round = 0_usize;
        loop {
            round = round.saturating_add(1);
            let execution_progress = self.advance_intent_execution();
            let product_progress = self.poll_intent_action_port();
            if self.terminal.is_stopped()
                || self.pending_managed_rebind.is_some()
                || self.pending_frame_presentation.is_some()
            {
                return PlatformPulseIntentProductCycleOutcome::Interrupted { rounds: round };
            }
            if let Some(completed) = execution_progress.complete_cycle(product_progress, round) {
                return completed;
            }
        }
    }
}

impl PlatformPulseIntentExecutionProgress {
    pub(super) const fn from_transitions(transitions: usize, locally_progressed: bool) -> Self {
        match (transitions, locally_progressed) {
            (0, _) => Self::Idle,
            (_, true) => Self::Progressed,
            (_, false) => Self::AwaitingExternal,
        }
    }

    const fn complete_cycle(
        self,
        product_progress: usize,
        rounds: usize,
    ) -> Option<PlatformPulseIntentProductCycleOutcome> {
        if product_progress != 0 {
            return None;
        }
        match self {
            Self::Idle => Some(PlatformPulseIntentProductCycleOutcome::Quiescent { rounds }),
            Self::AwaitingExternal => {
                Some(PlatformPulseIntentProductCycleOutcome::AwaitingExternal { rounds })
            }
            Self::Progressed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformPulseIntentExecutionProgress, PlatformPulseIntentProductCycleOutcome};

    #[test]
    fn pending_only_execution_is_typed_as_external_wait_instead_of_local_progress() {
        let progress = PlatformPulseIntentExecutionProgress::from_transitions(8, false);
        assert_eq!(
            progress,
            PlatformPulseIntentExecutionProgress::AwaitingExternal
        );
        assert_eq!(
            progress.complete_cycle(0, 1),
            Some(PlatformPulseIntentProductCycleOutcome::AwaitingExternal { rounds: 1 })
        );
        assert_eq!(progress.complete_cycle(1, 1), None);
    }

    #[test]
    fn terminal_or_started_execution_remains_local_progress() {
        assert_eq!(
            PlatformPulseIntentExecutionProgress::from_transitions(1, true),
            PlatformPulseIntentExecutionProgress::Progressed
        );
    }
}
