//! Owned workflow sessions: an owned session that also holds its workflow
//! Home's installed vocabulary, so a desktop value can run workflow reads and
//! lifecycle calls off the UI thread. Closing the Home releases both holds,
//! after which every call refuses `HomeClosed`.
use std::sync::{Arc, Mutex};

use worth_query_declaration::facade::{
    application_program::{ApplicationProgramDefinition, ApplicationWorkflowSpec},
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::application_installation::{
    WorthQueryWorkflowRuntimeBindingDenial, WorthQueryWorkflowRuntimeConfiguration,
    WorthQueryWorkflowVocabulary,
};

use super::{WorthQueryHostSession, WorthQueryOwnedSession, WorthQueryOwnedSessionDenial};
use crate::application_home::owned_holds::OwnedRuntimeHold;
use crate::application_home::WorthQueryWorkflowApplicationHome;
use crate::background::pending::WorthQueryPending;

/// A [`WorthQueryOwnedSession`] of a workflow Home, from
/// [`WorthQueryWorkflowApplicationHome::owned_session`].
///
/// It dereferences to the owned session for `run`, `query`, `spawn_*`,
/// `on_commit` and the rest. [`Self::with_workflow`] and
/// [`Self::spawn_with_workflow`] also lend the Home's installed workflow
/// vocabulary, bound to the runtime the call reads, which every workflow
/// discovery, page and lifecycle request requires.
///
/// ```
/// use worth_query_host::facade::{
///     declaration::{application_program::{ApplicationProgramDefinition, ApplicationWorkflowSpec},
///         application_schema::ApplicationSchema},
///     WorthQueryOwnedWorkflowSession,
/// };
/// fn refresh<S, Spec, P>(session: &WorthQueryOwnedWorkflowSession<S, Spec, P>, repaint: fn())
/// where S: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = S>,
///     P: ApplicationProgramDefinition<S> + 'static,
///     WorthQueryOwnedWorkflowSession<S, Spec, P>: Send {
///     let _pending = session
///         .spawn_with_workflow(|session, vocabulary| {
///             // Pass `vocabulary` to `session.request(&scope)` or to a
///             // `session.snapshot()?.with_request(..)` workflow read here.
///             let _ = vocabulary;
///             session.branch()
///         })
///         .with_waker(repaint);
/// }
/// ```
pub struct WorthQueryOwnedWorkflowSession<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    session: WorthQueryOwnedSession<Schema, Program>,
    workflow: Arc<OwnedWorkflowConfiguration<Schema, Spec>>,
}

/// The configuration an owned workflow session holds, `None` once its Home
/// closed. A call in flight keeps its own clone until it returns.
struct OwnedWorkflowConfiguration<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    configuration: Mutex<Option<Arc<WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>>>>,
}

impl<Schema, Spec> OwnedWorkflowConfiguration<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn attached(&self) -> Option<Arc<WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>>> {
        self.configuration
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl<Schema, Spec> OwnedRuntimeHold for OwnedWorkflowConfiguration<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn release(&self) {
        drop(
            self.configuration
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take(),
        );
    }
}

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>: Send + Sync,
{
    /// [`WorthQueryHostSession::into_owned`] for a session of this Home, also
    /// holding the installed workflow vocabulary until it is dropped or this
    /// Home closes. While one is attached, [`Self::support_workflow_spec`]
    /// refuses `RuntimeKeepaliveRetained`.
    ///
    /// A session of any other Home is returned unchanged.
    pub fn owned_session<'home>(
        &'home self,
        session: WorthQueryHostSession<'home, Schema, Program>,
    ) -> Result<
        WorthQueryOwnedWorkflowSession<Schema, Spec, Program>,
        WorthQueryHostSession<'home, Schema, Program>,
    > {
        let of_this_home = session
            .home()
            .is_some_and(|home| std::ptr::eq(home, &self.home));
        if !of_this_home {
            return Err(session);
        }
        let workflow = Arc::new(OwnedWorkflowConfiguration {
            configuration: Mutex::new(Some(Arc::clone(&self.configuration))),
        });
        let hold: Arc<dyn OwnedRuntimeHold + Send + Sync> = workflow.clone();
        self.home.owned.register_shared(Arc::downgrade(&hold));
        Ok(WorthQueryOwnedWorkflowSession {
            session: session.into_owned(),
            workflow,
        })
    }
}

impl<Schema, Spec, Program> WorthQueryOwnedWorkflowSession<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// [`WorthQueryOwnedSession::with_session`], also lending the installed
    /// workflow vocabulary bound to the same call's runtime.
    pub fn with_workflow<R>(
        &self,
        body: impl for<'call> FnOnce(
            &WorthQueryHostSession<'call, Schema, Program>,
            WorthQueryWorkflowVocabulary<'call, Schema, Spec>,
        ) -> R,
    ) -> Result<R, WorthQueryOwnedSessionDenial<WorthQueryWorkflowRuntimeBindingDenial>> {
        let configuration = self
            .workflow
            .attached()
            .ok_or(WorthQueryOwnedSessionDenial::HomeClosed)?;
        let lent = self.session.with_session(|session| {
            configuration
                .vocabulary_for(session.home.runtime())
                .map(|vocabulary| body(session, vocabulary))
        });
        match lent {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(denial)) => Err(WorthQueryOwnedSessionDenial::Denied(denial)),
            Err(WorthQueryOwnedSessionDenial::HomeClosed) => {
                Err(WorthQueryOwnedSessionDenial::HomeClosed)
            }
            Err(WorthQueryOwnedSessionDenial::Denied(never)) => match never {},
        }
    }

    /// [`Self::with_workflow`] on the session's call worker: a workflow read
    /// or lifecycle call that never blocks the calling thread. Its order and
    /// close behavior are those of [`WorthQueryOwnedSession::spawn_with_session`].
    pub fn spawn_with_workflow<R>(
        &self,
        body: impl for<'call> FnOnce(
                &WorthQueryHostSession<'call, Schema, Program>,
                WorthQueryWorkflowVocabulary<'call, Schema, Spec>,
            ) -> R
            + Send
            + 'static,
    ) -> WorthQueryPending<R, WorthQueryWorkflowRuntimeBindingDenial>
    where
        R: Send + 'static,
        Self: Send,
        WorthQueryOwnedSession<Schema, Program>: Send,
    {
        let session = self.clone();
        self.session.dispatch(move || session.with_workflow(body))
    }
}

impl<Schema, Spec, Program> std::ops::Deref
    for WorthQueryOwnedWorkflowSession<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    type Target = WorthQueryOwnedSession<Schema, Program>;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

impl<Schema, Spec, Program> Clone for WorthQueryOwnedWorkflowSession<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn clone(&self) -> Self {
        Self {
            session: self.session.clone(),
            workflow: Arc::clone(&self.workflow),
        }
    }
}
