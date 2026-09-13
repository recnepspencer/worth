use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeManagedSourceRebindOutcome,
    WorthUiNativeSourceRebindDenial,
};
use worth_ui::facade::rebind::{UiRebindReceipt, UiSourceRebindRequest};

pub(super) enum PlatformPulseRebindAction {
    Published(UiRebindReceipt),
    SourceDenied(WorthUiNativeSourceRebindDenial),
    Denied(WorthUiNativeSourceRebindDenial),
    Pending,
    Stopped(worth_ui::facade::app::WorthUiNativeManagedRebindStop),
}

pub(super) fn normalize_rebind(
    shell: &mut WorthUiNativeApplicationShell,
    snapshot: worth_ui::facade::source::WorthUiSettledSourceSnapshot,
    deadline_tick: u64,
    now_tick: u64,
) -> PlatformPulseRebindAction {
    let request = UiSourceRebindRequest::new(snapshot)
        .with_deadline(shell.rebind_deadline_at(deadline_tick))
        .observed_at_tick(now_tick);
    match shell
        .begin_source_rebind_with_layout(request, super::layout::prepare_replacement_native_layout)
    {
        Ok(WorthUiNativeManagedSourceRebindOutcome::Published(receipt)) => {
            PlatformPulseRebindAction::Published(receipt)
        }
        Ok(WorthUiNativeManagedSourceRebindOutcome::Pending) => PlatformPulseRebindAction::Pending,
        Ok(WorthUiNativeManagedSourceRebindOutcome::Stopped(stop)) => {
            PlatformPulseRebindAction::Stopped(stop)
        }
        Err(denial) if denial.source_failure().is_some() => {
            PlatformPulseRebindAction::SourceDenied(denial)
        }
        Err(denial) => PlatformPulseRebindAction::Denied(denial),
    }
}
