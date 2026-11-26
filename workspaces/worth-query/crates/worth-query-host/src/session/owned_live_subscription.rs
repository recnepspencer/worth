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

use super::live_subscription::WorthQueryTypedLiveSubscription;
use super::{QueryBinding, QueryMarker, QueryRow, WorthQueryHostSession};

/// A typed whole-result live subscription with no Home borrow.
///
/// Open it with [`WorthQueryHostSession::subscribe_owned`]. It has every
/// operation of [`WorthQueryTypedLiveSubscription`] (`current`, `refresh`,
/// `close`, ...) and may be stored in the same value as its Home, such as a
/// desktop application's state.
///
/// While it is open the Home's clean close and retirement refuse because its
/// runtime is still held; close or drop it first, then retire the Home.
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
        let owned = opened
            .into_owned(self.home.runtime().live_keepalive())
            .map_err(|_| WorthQueryApplicationLiveOpenRequestDenial::ForeignRuntime)?;
        Ok(WorthQueryTypedLiveSubscription::from_opened(owned))
    }
}
