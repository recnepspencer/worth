//! What a session reaches its Home through: the installed runtime and the
//! Home's owned-handle registry, borrowed from a Home or from an owned
//! session's runtime hold for the length of one call.
use std::sync::Arc;

use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};

use crate::application_home::owned_holds::OwnedRuntimeHolds;
use crate::application_home::WorthQueryApplicationHome;
use crate::background::commit_watch::CommitWatchRegistry;

pub(crate) struct SessionHome<'home, Schema, Program> {
    runtime: &'home WorthQueryProgramApplicationRuntime<Schema, Program>,
    holds: &'home Arc<OwnedRuntimeHolds>,
    commit_watch: &'home Arc<CommitWatchRegistry>,
    home: Option<&'home WorthQueryApplicationHome<Schema, Program>>,
}

impl<Schema, Program> Clone for SessionHome<'_, Schema, Program> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Program> Copy for SessionHome<'_, Schema, Program> {}

impl<'home, Schema, Program> SessionHome<'home, Schema, Program> {
    pub(crate) fn of_home(home: &'home WorthQueryApplicationHome<Schema, Program>) -> Self {
        Self {
            runtime: home.runtime(),
            holds: &home.owned,
            commit_watch: &home.commit_watch,
            home: Some(home),
        }
    }

    /// One call of an owned session over the runtime clone it holds.
    pub(crate) const fn of_runtime(
        runtime: &'home WorthQueryProgramApplicationRuntime<Schema, Program>,
        holds: &'home Arc<OwnedRuntimeHolds>,
        commit_watch: &'home Arc<CommitWatchRegistry>,
    ) -> Self {
        Self {
            runtime,
            holds,
            commit_watch,
            home: None,
        }
    }

    pub(crate) const fn runtime(
        &self,
    ) -> &'home WorthQueryProgramApplicationRuntime<Schema, Program> {
        self.runtime
    }

    pub(crate) fn primary_graph(&self) -> &'home WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime.runtime()
    }

    pub(crate) const fn holds(&self) -> &'home Arc<OwnedRuntimeHolds> {
        self.holds
    }

    pub(crate) const fn commit_watch(&self) -> &'home Arc<CommitWatchRegistry> {
        self.commit_watch
    }

    pub(crate) const fn home(&self) -> Option<&'home WorthQueryApplicationHome<Schema, Program>> {
        self.home
    }
}
