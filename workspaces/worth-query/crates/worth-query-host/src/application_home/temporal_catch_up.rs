//! Single-call caller-pumped temporal catch-up on [`WorthQueryApplicationHome`].

use worth_execution::ExecutionRequest;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalBinding;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryClockCoordinate};

use super::WorthQueryApplicationHome;
use crate::temporal_contribution::{
    pump_temporal_catch_up, pump_temporal_catch_up_until, WorthQueryDesktopTemporalHomeBinding,
    WorthQueryTemporalCatchUpDenial, WorthQueryTemporalCatchUpPolicy,
    WorthQueryTemporalCatchUpReport,
};

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
{
    /// Advances the caller-pumped clock for installed temporal binding `B` to
    /// `now` and pumps due timed intents on this Home's current world under
    /// `execution` and `policy` in one call.
    ///
    /// Strictly idempotent across close/reopen cycles and bounded by
    /// `execution` (`Work`/`Memory`/`Cancellation`) and `policy` (`max_steps`).
    /// OS background wake while the desktop application is closed remains
    /// outside WORTH.
    pub fn catch_up_temporal<B>(
        &self,
        now: WorthQueryClockCoordinate<
            <B::Installed as WorthQueryDesktopTemporalHomeBinding<Schema>>::Clock,
        >,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
    ) -> Result<
        WorthQueryTemporalCatchUpReport<
            <B::Installed as WorthQueryDesktopTemporalHomeBinding<Schema>>::Clock,
        >,
        WorthQueryTemporalCatchUpDenial,
    >
    where
        B: WorthQueryApplicationConditionalBinding<Schema> + 'static,
        B::Installed: WorthQueryDesktopTemporalHomeBinding<Schema>,
    {
        let Some(installed) = self.query.conditional::<B>() else {
            return Err(WorthQueryTemporalCatchUpDenial::TemporalBindingNotInstalled);
        };
        pump_temporal_catch_up_until(
            self.query.runtime(),
            self.query.current_world(),
            installed.clock_handle(),
            installed.desktop_clock(),
            now,
            execution,
            policy,
        )
    }

    /// Pumps due timed intents for installed temporal binding `B` at its
    /// currently admitted clock coordinate under `execution` and `policy`.
    pub fn pump_temporal<B>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
    ) -> Result<
        WorthQueryTemporalCatchUpReport<
            <B::Installed as WorthQueryDesktopTemporalHomeBinding<Schema>>::Clock,
        >,
        WorthQueryTemporalCatchUpDenial,
    >
    where
        B: WorthQueryApplicationConditionalBinding<Schema> + 'static,
        B::Installed: WorthQueryDesktopTemporalHomeBinding<Schema>,
    {
        let Some(installed) = self.query.conditional::<B>() else {
            return Err(WorthQueryTemporalCatchUpDenial::TemporalBindingNotInstalled);
        };
        pump_temporal_catch_up(
            self.query.runtime(),
            self.query.current_world(),
            installed.clock_handle(),
            execution,
            policy,
        )
    }
}
