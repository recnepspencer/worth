//! Caller-pumped temporal catch-up (SLAs, recurring rules) of the paired
//! Home, on its workflow Home.
//!
//! Each method is the same call on [`WorthQueryApplicationHome`] with the same
//! signature, pumping due timed intents on the Home's current world.
use super::*;
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::{
    application_contribution::WorthQueryApplicationConditionalBinding,
    primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
};
use worth_query_installation::facade::WorthQueryClockCoordinate;

use crate::temporal_contribution::{
    WorthQueryDesktopTemporalHomeBinding, WorthQueryTemporalCatchUpDenial,
    WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpReport,
};

type BindingClock<Schema, B> = <<B as WorthQueryApplicationConditionalBinding<Schema>>::Installed as WorthQueryDesktopTemporalHomeBinding<Schema>>::Clock;
type CatchUp<Schema, B> = Result<
    WorthQueryTemporalCatchUpReport<BindingClock<Schema, B>>,
    WorthQueryTemporalCatchUpDenial,
>;

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// [`WorthQueryApplicationHome::catch_up_temporal`].
    pub fn catch_up_temporal<B>(
        &self,
        now: WorthQueryClockCoordinate<BindingClock<Schema, B>>,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
    ) -> CatchUp<Schema, B>
    where
        B: WorthQueryApplicationConditionalBinding<Schema> + 'static,
        B::Installed: WorthQueryDesktopTemporalHomeBinding<Schema>,
    {
        self.home.catch_up_temporal::<B>(now, execution, policy)
    }

    /// [`WorthQueryApplicationHome::catch_up_temporal_observing`].
    pub fn catch_up_temporal_observing<B>(
        &self,
        now: WorthQueryClockCoordinate<BindingClock<Schema, B>>,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
        on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
    ) -> CatchUp<Schema, B>
    where
        B: WorthQueryApplicationConditionalBinding<Schema> + 'static,
        B::Installed: WorthQueryDesktopTemporalHomeBinding<Schema>,
    {
        self.home
            .catch_up_temporal_observing::<B>(now, execution, policy, on_invalidations)
    }

    /// [`WorthQueryApplicationHome::pump_temporal`].
    pub fn pump_temporal<B>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
    ) -> CatchUp<Schema, B>
    where
        B: WorthQueryApplicationConditionalBinding<Schema> + 'static,
        B::Installed: WorthQueryDesktopTemporalHomeBinding<Schema>,
    {
        self.home.pump_temporal::<B>(execution, policy)
    }

    /// [`WorthQueryApplicationHome::pump_temporal_observing`].
    pub fn pump_temporal_observing<B>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        policy: WorthQueryTemporalCatchUpPolicy,
        on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
    ) -> CatchUp<Schema, B>
    where
        B: WorthQueryApplicationConditionalBinding<Schema> + 'static,
        B::Installed: WorthQueryDesktopTemporalHomeBinding<Schema>,
    {
        self.home
            .pump_temporal_observing::<B>(execution, policy, on_invalidations)
    }
}
