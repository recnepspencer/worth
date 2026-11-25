//! Single-call caller-pumped temporal catch-up on [`WorthQueryApplicationHome`].

use worth_execution::ExecutionRequest;
use worth_query_execution::facade::product::WorthQueryProductBranch;
use worth_query_execution::facade::{
    application_contribution::WorthQueryApplicationConditionalBinding,
    primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryClockCoordinate};

use super::WorthQueryApplicationHome;
use crate::temporal_contribution::{
    pump_temporal_catch_up, pump_temporal_catch_up_observing, pump_temporal_catch_up_until,
    pump_temporal_catch_up_until_observing, WorthQueryDesktopTemporalHomeBinding,
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
    /// Non-empty granular invalidation delivery batches remain attached to the
    /// returned report via `take_granular_invalidation_batches()`; consume or
    /// drop the report before `close()` (or use [`Self::catch_up_temporal_observing`]
    /// to stream deliveries directly without retaining Home custody on the report).
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
        self.catch_up_temporal_on::<B>(self.query.current_world(), now, execution, policy)
    }

    /// [`Self::catch_up_temporal`] on `branch` rather than the current world.
    pub(crate) fn catch_up_temporal_on<B>(
        &self,
        branch: WorthQueryProductBranch,
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
            branch,
            installed.as_ref(),
            installed.desktop_clock(),
            now,
            execution,
            policy,
        )
    }

    /// Streaming variant of [`Self::catch_up_temporal`] forwarding every
    /// non-empty [`WorthQueryGranularInvalidationDeliveryBatch`] to
    /// `on_invalidations` immediately with O(1) accumulator memory.
    pub fn catch_up_temporal_observing<B>(
        &self,
        now: WorthQueryClockCoordinate<
            <B::Installed as WorthQueryDesktopTemporalHomeBinding<Schema>>::Clock,
        >,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
        on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
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
        pump_temporal_catch_up_until_observing(
            self.query.runtime(),
            self.query.current_world(),
            installed.as_ref(),
            installed.desktop_clock(),
            now,
            execution,
            policy,
            on_invalidations,
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

    /// Streaming variant of [`Self::pump_temporal`] delivering each step's
    /// non-empty granular invalidation batch directly to `on_invalidations`.
    pub fn pump_temporal_observing<B>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
        on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
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
        pump_temporal_catch_up_observing(
            self.query.runtime(),
            self.query.current_world(),
            installed.clock_handle(),
            execution,
            policy,
            on_invalidations,
        )
    }
}
