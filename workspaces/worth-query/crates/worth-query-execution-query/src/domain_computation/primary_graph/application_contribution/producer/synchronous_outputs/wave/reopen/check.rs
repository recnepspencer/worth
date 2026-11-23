//! What a reopened row selected before, merged across reopens, and the
//! routes and parents it is verified against.
use super::*;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// Record what `row` selected before a reopen at `invalidator` (none for
    /// a cleared capture). A row already reopened keeps what it selected
    /// before its first reopen: the checks merge (earliest invalidator, every
    /// prior child and parent), so a second reopen in the same edit never
    /// forgets a selection to verify.
    pub(in super::super) fn merge_reopen_check(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        row: usize,
        invalidator: Option<usize>,
        prior_children: Vec<SelectedRoute>,
        prior_prerequisites: Vec<usize>,
    ) -> Result<(), SynchronousOutputFailure> {
        if let Some(check) = &self.tasks[row].reopen_check {
            let work = check.prior_children.len().max(1) * prior_children.len()
                + check.prior_prerequisites.len()
                + prior_prerequisites.len();
            checkpoint(context.execution, self.tasks[row].subject, work as u64 + 1)?;
        }
        let check = match self.tasks[row].reopen_check.take() {
            None => ReopenCheck {
                invalidator,
                prior_children,
                prior_prerequisites,
            },
            Some(mut check) => {
                check.invalidator = match (check.invalidator, invalidator) {
                    (Some(earlier), Some(later)) => Some(earlier.min(later)),
                    (earlier, later) => earlier.or(later),
                };
                for child in prior_children {
                    if !check.prior_children.contains(&child) {
                        check.prior_children.push(child);
                    }
                }
                check.prior_prerequisites.extend(prior_prerequisites);
                check.prior_prerequisites.sort_unstable();
                check.prior_prerequisites.dedup();
                check
            }
        };
        self.tasks[row].reopen_check = Some(check);
        Ok(())
    }

    /// The routes of `rows`: an occurrence's key, otherwise its route read.
    pub(super) fn routes(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        rows: Vec<usize>,
    ) -> Result<Vec<SelectedRoute>, SynchronousOutputFailure> {
        let mut routes = Vec::new();
        let mut memory = None;
        for row in rows {
            let subject = self.tasks[row].subject;
            checkpoint(context.execution, subject, 1)?;
            let route = match self.tasks[row].key.clone() {
                Some(key) => key,
                None => self.tasks[row].tree.route(context, candidate)?,
            };
            state::push_funded(
                &mut routes,
                &mut memory,
                (subject, route),
                context.execution,
                subject,
            )?;
        }
        Ok(routes)
    }

    /// Whether task `t` derived in this edit: done, or done before and
    /// reopened since (its earlier part is still in the candidate).
    pub(in super::super) fn has_derived(&self, t: usize) -> bool {
        self.tasks[t].is_done() || self.tasks[t].reopened
    }

    /// The canonical prerequisites of `i` a re-closure must select again:
    /// each that derived in this edit, even if reopened since (plan §1.6:
    /// only a selection dropped after it derived refuses). A derived task
    /// orphaned any other way is the quiescence backstop's
    /// (`require_no_orphans`).
    pub(in super::super) fn derived_selections(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
    ) -> Result<Vec<usize>, SynchronousOutputFailure> {
        let set = self.canonical_set(i);
        checkpoint(context.execution, self.tasks[i].subject, set.len() as u64 + 1)?;
        Ok(set.into_iter().filter(|&parent| self.has_derived(parent)).collect())
    }

    /// The canonical prerequisites of `i`, sorted.
    pub(in super::super) fn canonical_set(&self, i: usize) -> Vec<usize> {
        let mut set = self.tasks[i]
            .prerequisites
            .iter()
            .map(|parent| self.canonical(*parent))
            .collect::<Vec<_>>();
        set.sort_unstable();
        set.dedup();
        set
    }
}
