//! Installed Source-scoped rules use Native's complete invocation closure.
use super::{CustomInvariantExecutionContext, CustomInvariantScopePlanner};
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial, ManagedIndexSourceProgram, ManagedIndexSourceRead,
    PreparedManagedIndexSourceRead,
};

/// Set only by the installed Native scheduler; entity identity alone is not an audience.
#[derive(Clone)]
pub(super) struct ManagedSourceInvocation {
    pub(super) scope: crate::identity::data::EntityId,
    pub(super) sources: std::sync::Arc<[ManagedIndexSourceProgram]>,
    pub(super) audience: crate::validation::data::CustomInvariantSemanticIdentity,
}

impl CustomInvariantScopePlanner<'_> {
    /// Only the Native scheduler sets an invocation scope. The installed Source
    /// program and the exact actual observation must both admit its root.
    pub fn prepare_managed_index_source(
        &self,
        program: &ManagedIndexSourceProgram,
    ) -> Result<PreparedManagedIndexSourceRead, ManagedIndexSourcePreparationDenial> {
        let invocation = self
            .managed_scope
            .as_ref()
            .ok_or(ManagedIndexSourcePreparationDenial::ForeignSource)?;
        if !invocation.sources.contains(program) {
            return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
        }
        crate::validation::candidate_source::prepare_observed_root(
            program,
            invocation.scope,
            &invocation.audience,
            &self.committed_relations(),
            &self.relations(),
        )
    }
}

impl<'runtime> CustomInvariantExecutionContext<'runtime> {
    /// The prepared object carries no read grant. Reborrow checks this actual
    /// observation, receiving audience and original request before returning data.
    pub fn read_managed_index_source<'a>(
        &'a self,
        prepared: &'a PreparedManagedIndexSourceRead,
    ) -> Result<ManagedIndexSourceRead<'a>, ManagedIndexSourcePreparationDenial> {
        let invocation = self
            .managed_scope
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or(ManagedIndexSourcePreparationDenial::ForeignSource)?;
        if !prepared.matches_invocation(invocation.scope, &invocation.sources, &invocation.audience)
        {
            return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
        }
        crate::validation::candidate_source::read_observed_root(
            prepared,
            crate::validation::candidate_source::SourceViews {
                before: self.committed_relations(),
                after: self.relations(),
                before_fields: self.committed_aspect_states(),
                after_fields: self.aspect_states(),
            },
        )
    }

    pub(super) fn in_managed_source_invocation<T>(
        &self,
        invocation: ManagedSourceInvocation,
        run: impl FnOnce(&Self) -> T,
    ) -> T {
        let previous = self
            .managed_scope
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(invocation);
        let _restore = InvocationRestore {
            context: self,
            previous,
        };
        run(self)
    }
}

struct InvocationRestore<'a, 'runtime> {
    context: &'a CustomInvariantExecutionContext<'runtime>,
    previous: Option<ManagedSourceInvocation>,
}

impl Drop for InvocationRestore<'_, '_> {
    fn drop(&mut self) {
        *self
            .context
            .managed_scope
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = self.previous.take();
    }
}
