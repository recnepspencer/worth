//! Non-blocking owned-session calls: each `spawn_*` queues the call on the
//! session's call worker and returns a [`WorthQueryPending`] at once, so a UI
//! thread never waits on a commit or a read.
//!
//! Calls run one at a time in the order queued (commits never interleave out
//! of order), each exactly as the blocking method would run it. A Home closed
//! before a call runs makes it refuse `HomeClosed`; a Home closed while a call
//! runs refuses that close until the call returns (retry it, or `wait` first)
//! and never blocks.
use std::convert::Infallible;
use std::panic::{catch_unwind, AssertUnwindSafe};

use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
        NoApplicationMutationSource,
    },
    application_program::ApplicationProgramDefinition,
    application_schema::ApplicationSchema,
};
use worth_query_publication::facade::application_entry::WorthQueryApplicationRequestQueryDenial;

use super::{
    CallWorker, WorthQueryAutoIdempotencyKey, WorthQueryHostSession, WorthQueryOwnedSession,
    WorthQueryOwnedSessionDenial, WorthQuerySessionCommit, WorthQuerySessionQuery,
    WorthQuerySessionRunDenial,
};
use crate::background::call_worker::BackgroundCall;
use crate::background::pending::{pending, WorthQueryBackgroundDenial, WorthQueryPending};

type MutationResult<Schema, Intent> =
    <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::Result;
type MutationDenial<Schema, Intent> =
    <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::Denial;

impl<Schema, Program> WorthQueryOwnedSession<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Self: Send,
{
    /// [`Self::run`] on the session's call worker.
    pub fn spawn_run<Intent>(
        &self,
        intent: Intent,
    ) -> WorthQueryPending<
        WorthQuerySessionCommit<MutationResult<Schema, Intent>>,
        WorthQuerySessionRunDenial<MutationDenial<Schema, Intent>>,
    >
    where
        Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync + 'static,
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
        MutationResult<Schema, Intent>: Send + 'static,
        MutationDenial<Schema, Intent>: Send + 'static,
    {
        let session = self.clone();
        self.dispatch(move || session.run(intent))
    }

    /// [`Self::query`] on the session's call worker.
    pub fn spawn_query<Intent>(
        &self,
        intent: Intent,
    ) -> WorthQueryPending<Vec<Intent::Row>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: WorthQuerySessionQuery<Schema> + Send + 'static,
        Intent::Row: Send + 'static,
    {
        let session = self.clone();
        self.dispatch(move || session.query(intent))
    }

    /// [`Self::with_session`] on the session's call worker, for any other
    /// session call (a snapshot's queries, a paged walk, `query_result`, ...).
    pub fn spawn_with_session<R>(
        &self,
        body: impl FnOnce(&WorthQueryHostSession<'_, Schema, Program>) -> R + Send + 'static,
    ) -> WorthQueryPending<R, Infallible>
    where
        R: Send + 'static,
    {
        let session = self.clone();
        self.dispatch(move || session.with_session(body))
    }

    /// Queues `call`, completing its handle with the call's outcome, a
    /// contained panic, or why it never ran.
    fn dispatch<T, D>(
        &self,
        call: impl FnOnce() -> Result<T, WorthQueryOwnedSessionDenial<D>> + Send + 'static,
    ) -> WorthQueryPending<T, D>
    where
        T: Send + 'static,
        D: Send + 'static,
    {
        let (pending, completer) = pending();
        let queued: BackgroundCall = Box::new(move || {
            let outcome = match catch_unwind(AssertUnwindSafe(call)) {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(WorthQueryOwnedSessionDenial::HomeClosed)) => {
                    Err(WorthQueryBackgroundDenial::HomeClosed)
                }
                Ok(Err(WorthQueryOwnedSessionDenial::Denied(denial))) => {
                    Err(WorthQueryBackgroundDenial::Denied(denial))
                }
                Err(_) => Err(WorthQueryBackgroundDenial::Panicked),
            };
            completer.complete(outcome);
        });
        self.custody
            .calls
            .get_or_init(CallWorker::start)
            .submit(&pending, queued);
        pending
    }
}
