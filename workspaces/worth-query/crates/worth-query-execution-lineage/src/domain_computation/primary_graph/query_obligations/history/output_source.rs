//! Lifecycle records use the one serialized Query publisher.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableCompositeCommitIdentity;
use worth_query_execution_obligations::facade::ledger_inspect;

impl QueryObligationHistory {
    /// Same canonical source-root meaning and scope; liveness belongs to the
    /// source registry, which decides whether each candidate may be retired.
    pub fn discovered_source_predecessors(
        &self,
        current: DurableCompositeCommitIdentity,
    ) -> Vec<DurableCompositeCommitIdentity> {
        let state = self.state();
        let Some(output) = state
            .journal
            .as_ref()
            .map(|journal| ledger_inspect::output(&journal.ledger))
        else {
            return Vec::new();
        };
        let Some(current_row) = output.sources.get(&current) else {
            return Vec::new();
        };
        output
            .sources
            .values()
            .filter(|prior| {
                prior.discovery_recipe.is_some()
                    && prior.commit.ordinal() < current.ordinal()
                    && prior.commit.owner() == current.owner()
                    && prior.revision == current_row.revision
                    && prior.root == current_row.root
                    && prior.branch == current_row.branch
                    && prior.lifecycle == current_row.lifecycle
                    && prior.scope == current_row.scope
                    && !output.demands.values().any(|demand| {
                        !output.completed.contains(&demand.acquire_position)
                            && demand.performed_sources.contains(&prior.commit)
                    })
            })
            .map(|prior| prior.commit)
            .collect()
    }
    pub fn output_sources(&self) -> Vec<QueryOutputSourceObligation> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| {
                ledger_inspect::output(&journal.ledger)
                    .sources
                    .values()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn active_output_sources_for_branch(
        &self,
        branch: &str,
        lifecycle: u64,
    ) -> Vec<DurableCompositeCommitIdentity> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| {
                ledger_inspect::output(&journal.ledger)
                    .sources
                    .values()
                    .filter(|row| row.branch == branch && row.lifecycle == lifecycle)
                    .map(|row| row.commit)
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn retain_output_source(
        &self,
        request: &ExecutionRequest<'_, '_>,
        row: QueryOutputSourceObligation,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if self
            .state()
            .retired_branches
            .contains(&(row.branch.clone(), row.lifecycle))
        {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        if let Some(prior) = self.state().journal.as_ref().and_then(|journal| {
            ledger_inspect::output(&journal.ledger)
                .sources
                .get(&row.commit)
        }) {
            return if prior == &row {
                Ok(())
            } else {
                Err(QueryObligationDenial::ObligationSubjectMismatch)
            };
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputSourceRecord::Retain(row),
            request,
        )
    }
    pub fn discharge_output_source(
        &self,
        request: &ExecutionRequest<'_, '_>,
        commit: DurableCompositeCommitIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if !self.state().journal.as_ref().is_some_and(|journal| {
            ledger_inspect::output(&journal.ledger)
                .sources
                .contains_key(&commit)
        }) {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputSourceRecord::Discharge { commit },
            request,
        )
    }
    pub fn bind_output_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        commit: DurableCompositeCommitIdentity,
        mut sources: Vec<([u8; 32], (u32, u64, u32))>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        sources.sort_unstable();
        sources.dedup();
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), execution)?;
        let prior = self
            .state()
            .journal
            .as_ref()
            .and_then(|journal| {
                ledger_inspect::output(&journal.ledger)
                    .sources
                    .get(&commit)
                    .cloned()
            })
            .ok_or(QueryObligationDenial::UnknownObligation)?;
        if let Some(known) = prior.bindings {
            return if known == sources {
                Ok(())
            } else {
                Err(QueryObligationDenial::ObligationSubjectMismatch)
            };
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputSourceRecord::Bind { commit, sources },
            execution,
        )
    }
}
