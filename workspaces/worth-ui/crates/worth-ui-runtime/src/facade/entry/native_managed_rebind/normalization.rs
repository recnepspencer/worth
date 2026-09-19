use super::{
    WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindStop,
    WorthUiNativePendingManagedRebind,
};

pub(crate) enum ManagedRebindNormalization {
    Indeterminate {
        recovery: crate::runtime::rebind::UiDetachedRebindRecovery,
        frame: crate::mounting::UiMountedIndeterminateFrame,
    },
    Published(crate::runtime::rebind::UiRebindReceipt),
    Pending(crate::runtime::rebind::UiDetachedRebindCompletion),
    Retry {
        retry: crate::runtime::rebind::UiDetachedRebindRetry,
        requires_reconstruction: bool,
    },
    Stopped(WorthUiNativeManagedRebindStop),
}

pub(crate) fn normalize_managed_outcome(
    outcome: crate::runtime::rebind::UiRebindOutcome<'_>,
) -> ManagedRebindNormalization {
    use crate::runtime::rebind::UiRebindOutcome;
    match outcome {
        UiRebindOutcome::Published(receipt) => ManagedRebindNormalization::Published(receipt),
        UiRebindOutcome::InFlight(completion) => {
            ManagedRebindNormalization::Pending(completion.detach_for_native())
        }
        UiRebindOutcome::Duplicate(_) => {
            ManagedRebindNormalization::Stopped(WorthUiNativeManagedRebindStop::Duplicate)
        }
        UiRebindOutcome::ObservedNoChange(_) => {
            ManagedRebindNormalization::Stopped(WorthUiNativeManagedRebindStop::ObservedNoChange)
        }
        UiRebindOutcome::RejectedBeforeEffects(denial) => {
            let host_denials = denial
                .host_rejections()
                .iter()
                .map(|rejection| rejection.denial())
                .collect::<Vec<_>>()
                .into_boxed_slice();
            let requires_reconstruction = !host_denials.is_empty()
                && host_denials.iter().all(|denial| {
                    *denial
                        == worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired
                });
            let phase = denial.stopped_phase();
            let cause = denial.cause();
            if let Ok(retry) = denial.detach_retry_for_native() {
                return ManagedRebindNormalization::Retry {
                    retry,
                    requires_reconstruction,
                };
            }
            ManagedRebindNormalization::Stopped(
                WorthUiNativeManagedRebindStop::RejectedBeforeEffects {
                    phase,
                    cause,
                    host_denials,
                },
            )
        }
        UiRebindOutcome::CancelledBeforeEffects(receipt) => ManagedRebindNormalization::Stopped(
            WorthUiNativeManagedRebindStop::CancelledBeforeEffects(receipt),
        ),
        UiRebindOutcome::TimedOutBeforeEffects(receipt) => ManagedRebindNormalization::Stopped(
            WorthUiNativeManagedRebindStop::TimedOutBeforeEffects(receipt),
        ),
        UiRebindOutcome::SupersededBeforeEffects(receipt) => ManagedRebindNormalization::Stopped(
            WorthUiNativeManagedRebindStop::SupersededBeforeEffects(receipt),
        ),
        UiRebindOutcome::Indeterminate(recovery) => {
            let (recovery, frame) = recovery.detach_for_native();
            ManagedRebindNormalization::Indeterminate { recovery, frame }
        }
        UiRebindOutcome::InternalDefect(defect) => ManagedRebindNormalization::Stopped(
            WorthUiNativeManagedRebindStop::InternalDefect(defect.kind()),
        ),
    }
}

pub(crate) fn finish_normalized_managed_rebind(
    pending: &mut Option<WorthUiNativePendingManagedRebind>,
    outcome: crate::runtime::rebind::UiRebindOutcome<'_>,
) -> WorthUiNativeManagedRebindProgress {
    retain_normalized_managed_rebind(pending, normalize_managed_outcome(outcome))
}

pub(crate) fn retain_normalized_managed_rebind(
    pending: &mut Option<WorthUiNativePendingManagedRebind>,
    normalized: ManagedRebindNormalization,
) -> WorthUiNativeManagedRebindProgress {
    match normalized {
        ManagedRebindNormalization::Indeterminate { recovery, frame } => {
            *pending = Some(WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame });
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        }
        ManagedRebindNormalization::Published(receipt) => {
            WorthUiNativeManagedRebindProgress::Published(receipt)
        }
        ManagedRebindNormalization::Pending(completion) => {
            *pending = Some(WorthUiNativePendingManagedRebind::Completion(completion));
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        }
        ManagedRebindNormalization::Retry {
            retry,
            requires_reconstruction,
        } => {
            *pending = Some(WorthUiNativePendingManagedRebind::Retry {
                retry,
                requires_reconstruction,
            });
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        }
        ManagedRebindNormalization::Stopped(stop) => {
            WorthUiNativeManagedRebindProgress::Stopped(stop)
        }
    }
}

pub(super) fn retry_progressed_text_atlas_deferral<'session>(
    outcome: crate::runtime::rebind::UiRebindOutcome<'session>,
    now_tick: u64,
) -> crate::runtime::rebind::UiRebindOutcome<'session> {
    let crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) = outcome else {
        return outcome;
    };
    let rejections = denial.host_rejections();
    if !rejections.is_empty()
        && rejections.iter().all(|rejection| {
            rejection.denial()
                == worth_ui_host_contract::UiHostSurfacePresentationDenial::
                    TextAtlasPresentationDeferred
        })
    {
        denial.retry_at(now_tick)
    } else {
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial)
    }
}
