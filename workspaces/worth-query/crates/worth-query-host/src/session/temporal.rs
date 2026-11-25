//! Bounded temporal catch-up funded by the session's execution policy.
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalBinding;
use worth_query_installation::facade::WorthQueryClockCoordinate;

use super::WorthQueryHostSession;
use crate::temporal_contribution::{
    WorthQueryDesktopTemporalHomeBinding, WorthQueryTemporalCatchUpDenial,
    WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpReport,
};

type BindingClock<Schema, Binding> =
    <<Binding as WorthQueryApplicationConditionalBinding<Schema>>::Installed as WorthQueryDesktopTemporalHomeBinding<Schema>>::Clock;

impl<Schema: ApplicationSchema + 'static, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Advances installed temporal binding `Binding`'s caller-pumped clock to
    /// `now` and fires due timed intents on this session's branch, funding the
    /// run from this session's execution policy instead of a caller-assembled
    /// lease.
    ///
    /// `policy` bounds the number of steps; the report is that of
    /// [`crate::application_home::WorthQueryApplicationHome::catch_up_temporal`].
    pub fn catch_up_temporal<Binding>(
        &self,
        now: WorthQueryClockCoordinate<BindingClock<Schema, Binding>>,
        policy: WorthQueryTemporalCatchUpPolicy,
    ) -> Result<
        WorthQueryTemporalCatchUpReport<BindingClock<Schema, Binding>>,
        WorthQueryTemporalCatchUpDenial,
    >
    where
        Binding: WorthQueryApplicationConditionalBinding<Schema> + 'static,
        Binding::Installed: WorthQueryDesktopTemporalHomeBinding<Schema>,
    {
        let scope = self.fresh_scope();
        let lease = self
            .open_lease(&scope)
            .map_err(WorthQueryTemporalCatchUpDenial::Advancement)?;
        self.home.catch_up_temporal_on::<Binding>(
            self.branch,
            now,
            &ExecutionRequest::leased(&lease),
            policy,
        )
    }
}
