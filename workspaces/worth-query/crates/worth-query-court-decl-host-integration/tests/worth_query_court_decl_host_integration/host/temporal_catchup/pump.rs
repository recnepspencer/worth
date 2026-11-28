//! Drives the installed reminder clock through the temporal contribution API.
use super::{program::ReminderHome, *};
use worth_query_host::facade::{
    primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
    temporal_contribution::{
        pump_temporal_catch_up, pump_temporal_catch_up_observing, pump_temporal_catch_up_until,
        pump_temporal_catch_up_until_observing, WorthQueryDesktopTemporalHomeBinding,
    },
};

type Report =
    Result<WorthQueryTemporalCatchUpReport<DesktopReminderClock>, WorthQueryTemporalCatchUpDenial>;

pub(super) fn catch_up(
    home: &ReminderHome,
    now: WorthQueryClockCoordinate<DesktopReminderClock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
) -> Report {
    let installed = installed(home);
    pump_temporal_catch_up_until(
        home.primary_graph(),
        home.runtime().current_world(),
        installed.as_ref(),
        installed.desktop_clock(),
        now,
        execution,
        policy,
    )
}

pub(super) fn catch_up_observing(
    home: &ReminderHome,
    now: WorthQueryClockCoordinate<DesktopReminderClock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
    on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
) -> Report {
    let installed = installed(home);
    pump_temporal_catch_up_until_observing(
        home.primary_graph(),
        home.runtime().current_world(),
        installed.as_ref(),
        installed.desktop_clock(),
        now,
        execution,
        policy,
        on_invalidations,
    )
}

pub(super) fn pump(
    home: &ReminderHome,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
) -> Report {
    pump_temporal_catch_up(
        home.primary_graph(),
        home.runtime().current_world(),
        installed(home).clock_handle(),
        execution,
        policy,
    )
}

pub(super) fn pump_observing(
    home: &ReminderHome,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
    on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
) -> Report {
    pump_temporal_catch_up_observing(
        home.primary_graph(),
        home.runtime().current_world(),
        installed(home).clock_handle(),
        execution,
        policy,
        on_invalidations,
    )
}
