//! Owned `'static` sessions: a session that holds its Home's runtime instead of
//! borrowing the Home, so a desktop or GUI value can store the Home, its
//! session and its owned live subscriptions side by side. Closing the Home
//! releases the session, after which every call refuses `HomeClosed`.
use std::sync::{Arc, Mutex};

use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
        NoApplicationMutationSource,
    },
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::WorthQueryApplicationProjection,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationLiveOpenRequestDenial, WorthQueryApplicationRequestQueryDenial,
};

use super::{
    QueryBinding, QueryMarker, QueryRow, SessionHome, SessionSettings,
    WorthQueryAutoIdempotencyKey, WorthQueryHostSession, WorthQueryOwnedLiveSubscription,
    WorthQuerySessionCommit, WorthQuerySessionQuery, WorthQuerySessionRunDenial,
};
use crate::application_home::owned_holds::{OwnedRuntimeHold, OwnedRuntimeHolds};

/// A call of an owned session refused, or its Home closed first.
#[derive(Debug)]
pub enum WorthQueryOwnedSessionDenial<Denial> {
    /// The Home closed and released this session; open a new Home and session.
    HomeClosed,
    Denied(Denial),
}

/// A session with no Home borrow, from [`WorthQueryHostSession::into_owned`].
///
/// Clones share one hold on the Home's runtime. [`Self::with_session`] lends
/// the complete borrowed API for one call; `run`, `query` and `subscribe`
/// cover the common calls directly. A renewing session keeps renewing.
pub struct WorthQueryOwnedSession<Schema: ApplicationSchema, Program> {
    custody: Arc<OwnedRuntime<Schema, Program>>,
    holds: Arc<OwnedRuntimeHolds>,
    settings: SessionSettings<Schema>,
}

/// The runtime clone an owned session holds, `None` once its Home closed.
struct OwnedRuntime<Schema, Program> {
    runtime: Mutex<Option<WorthQueryProgramApplicationRuntime<Schema, Program>>>,
}

impl<Schema, Program> OwnedRuntimeHold for OwnedRuntime<Schema, Program> {
    fn release(&self) {
        drop(
            self.runtime
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take(),
        );
    }
}

impl<Schema: ApplicationSchema, Program> Clone for WorthQueryOwnedSession<Schema, Program> {
    fn clone(&self) -> Self {
        Self {
            custody: Arc::clone(&self.custody),
            holds: Arc::clone(&self.holds),
            settings: self.settings.clone(),
        }
    }
}

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// The same session (principal, renewal, branch, policy and limits) as an
    /// owned `'static` value that holds the Home's runtime until either it is
    /// dropped or the Home closes.
    pub fn into_owned(self) -> WorthQueryOwnedSession<Schema, Program> {
        let custody = Arc::new(OwnedRuntime {
            runtime: Mutex::new(Some(self.home.runtime().clone())),
        });
        let hold: Arc<dyn OwnedRuntimeHold + Send + Sync> = custody.clone();
        self.home.holds().register_shared(Arc::downgrade(&hold));
        WorthQueryOwnedSession {
            custody,
            holds: Arc::clone(self.home.holds()),
            settings: self.settings,
        }
    }
}

impl<Schema, Program> WorthQueryOwnedSession<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// Whether the Home is still open for this session.
    pub fn is_attached(&self) -> bool {
        self.attached_runtime().is_some()
    }

    fn attached_runtime(&self) -> Option<WorthQueryProgramApplicationRuntime<Schema, Program>> {
        self.custody
            .runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Lends this session as a borrowed [`WorthQueryHostSession`] for one call.
    ///
    /// The call holds its own runtime clone, so a Home retired from inside
    /// `body` refuses until `body` returns rather than blocking.
    pub fn with_session<R>(
        &self,
        body: impl FnOnce(&WorthQueryHostSession<'_, Schema, Program>) -> R,
    ) -> Result<R, WorthQueryOwnedSessionDenial<std::convert::Infallible>> {
        let runtime = self
            .attached_runtime()
            .ok_or(WorthQueryOwnedSessionDenial::HomeClosed)?;
        let session = WorthQueryHostSession {
            home: SessionHome::of_runtime(&runtime, &self.holds),
            settings: self.settings.clone(),
        };
        Ok(body(&session))
    }

    fn call<T, D>(
        &self,
        body: impl FnOnce(&WorthQueryHostSession<'_, Schema, Program>) -> Result<T, D>,
    ) -> Result<T, WorthQueryOwnedSessionDenial<D>> {
        match self.with_session(body) {
            Ok(outcome) => outcome.map_err(WorthQueryOwnedSessionDenial::Denied),
            Err(_) => Err(WorthQueryOwnedSessionDenial::HomeClosed),
        }
    }

    /// [`WorthQueryHostSession::run`] through this owned session.
    pub fn run<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQuerySessionCommit<<Intent::Binding as ApplicationMutationBinding<Schema>>::Result>,
        WorthQueryOwnedSessionDenial<
            WorthQuerySessionRunDenial<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            >,
        >,
    >
    where
        Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
        Intent::Binding:
            ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey:
            WorthQueryAutoIdempotencyKey,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        self.call(|session| session.run(intent))
    }

    /// [`WorthQueryHostSession::query`] through this owned session.
    pub fn query<Intent: WorthQuerySessionQuery<Schema>>(
        &self,
        intent: Intent,
    ) -> Result<
        Vec<Intent::Row>,
        WorthQueryOwnedSessionDenial<WorthQueryApplicationRequestQueryDenial>,
    > {
        self.call(|session| session.query(intent))
    }

    /// [`WorthQueryHostSession::subscribe_owned`] through this owned session.
    pub fn subscribe<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryOwnedLiveSubscription<Schema, Intent>,
        WorthQueryOwnedSessionDenial<WorthQueryApplicationLiveOpenRequestDenial>,
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
        self.call(|session| session.subscribe_owned(intent))
    }
}
