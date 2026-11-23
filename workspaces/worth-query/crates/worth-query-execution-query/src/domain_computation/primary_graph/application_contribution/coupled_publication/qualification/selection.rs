//! Qualified use of selections (PR C §1.6, #238): every append compares the
//! live query support a wave task's selector retained. A change ends that
//! support at the append, its authenticated invalidator, and the task selects
//! again before any later use, owed at the later of its own demand and the
//! invalidator (R1). A root selector's support holds only within its own
//! stage: an authored stage's append ends it there, never re-seeding it.
use super::*;

/// What qualified use reopens.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph::application_contribution) enum ReopenTarget {
    /// A retained parent output whose derivation must run again.
    Retention(ParentRead),
    /// The wave task whose selector's support changed: it selects again.
    /// `incoming` when its own prerequisite selection changed, so it derives
    /// again; otherwise only its children are selected again.
    Selection { task: usize, incoming: bool },
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// Compare every live task-owned selection support at the append at
    /// `invalidator`; each task whose support changed selects again.
    pub(super) fn mark_selections(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        invalidator: usize,
        invalidator_owed: usize,
        stage: bool,
    ) -> Result<(), CoupledSourceReadStop> {
        let provider = &self.application.runtime.primary_provider;
        if !provider
            .has_live_selection_support(&self.root_session)
            .map_err(CoupledSourceReadStop::capture)?
        {
            return Ok(());
        }
        let first_root = synchronous_outputs::SynchronousOutputWave::<S>::first_root_owner();
        if stage {
            // Every earlier root's stage ends at this authored stage's append.
            provider
                .end_root_selections(execution, &self.root_session, first_root, invalidator)
                .map_err(CoupledSourceReadStop::capture)?;
        }
        let current = candidate.pin_source(execution)?;
        let owners = provider
            .changed_selection_owners(
                execution,
                &self.root_session,
                self.before,
                &current,
                invalidator,
                first_root,
            )
            .map_err(CoupledSourceReadStop::capture)?;
        let mut reopens = self.reopens.borrow_mut();
        for (task, incoming) in owners {
            reopens.push(Reopen {
                target: ReopenTarget::Selection { task, incoming },
                invalidator,
                invalidator_owed,
            });
        }
        Ok(())
    }
}
