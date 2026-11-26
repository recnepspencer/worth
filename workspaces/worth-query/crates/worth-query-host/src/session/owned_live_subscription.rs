//! Owned typed live subscriptions that keep their Home's runtime alive, so a
//! desktop or GUI value can hold both the Home and its open subscriptions.
use worth_query_declaration::facade::{
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;
use worth_query_publication::facade::application_entry::WorthQueryApplicationLiveOpenRequestDenial;

use super::live_slot::{HeldLive, LiveSlot};
use super::live_subscription::{WorthQueryTypedLiveRefreshDenial, WorthQueryTypedLiveSubscription};
use super::owned_session::WorthQueryOwnedSession;
use super::{QueryBinding, QueryMarker, QueryRow, WorthQueryHostSession};

/// A typed whole-result live subscription with no Home borrow.
///
/// Open it with [`WorthQueryHostSession::subscribe_owned`]. It has every
/// operation of [`WorthQueryTypedLiveSubscription`] (`current`, `refresh`,
/// `close`, ...) and may be stored in the same value as its Home, such as a
/// desktop application's state.
///
/// Closing or retiring the Home on the thread that opened it closes it for
/// you: afterwards `refresh` refuses `Closed` and dropping it is harmless, so
/// no drop ordering is owed. Its native lease is pinned to that thread, so a
/// Home retired on another thread refuses until the subscription is dropped
/// or polled: [`Self::refresh_owned`] through a detached session closes it.
pub type WorthQueryOwnedLiveSubscription<Schema, Intent> =
    WorthQueryTypedLiveSubscription<'static, Schema, Intent>;

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// Opens a typed whole-result live subscription that keeps this Home's
    /// runtime alive instead of borrowing the Home.
    ///
    /// Refresh it with any session of the same Home; the subscription admits
    /// the principal and grants afresh on every poll, exactly like
    /// [`Self::subscribe`].
    pub fn subscribe_owned<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryOwnedLiveSubscription<Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        let opened = self.open_whole_result(intent)?;
        let mut owned = opened
            .into_owned(self.home.runtime().live_keepalive())
            .map_err(|_| WorthQueryApplicationLiveOpenRequestDenial::ForeignRuntime)?;
        let current = owned
            .take_initial()
            .expect("opened whole-result live subscription always captures initial snapshot");
        let held = HeldLive::new(owned, self.home.holds());
        Ok(WorthQueryTypedLiveSubscription::from_slot(
            LiveSlot::Held(held),
            current,
        ))
    }
}

impl<Schema, Intent> WorthQueryOwnedLiveSubscription<Schema, Intent>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    /// [`Self::refresh`] through an owned session. Once the session's Home has
    /// closed it refuses `Closed` and closes this subscription's lease, so a
    /// retirement retried from another thread then completes.
    pub fn refresh_owned<Program>(
        &mut self,
        session: &WorthQueryOwnedSession<Schema, Program>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial>
    where
        Program: ApplicationProgramDefinition<Schema> + 'static,
    {
        if !session.is_attached() {
            return self.released();
        }
        match session.with_session(|session| self.refresh(session)) {
            Ok(refreshed) => refreshed,
            Err(_) => Err(WorthQueryTypedLiveRefreshDenial::Closed),
        }
    }
}
