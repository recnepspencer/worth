//! Lifecycle records use the one serialized Query publisher.
use super::*;
use journal::*;
use std::num::NonZeroU64;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableCompositeCommitIdentity;
use worth_query_execution_obligations::facade::ledger_inspect;

fn merge_sorted<T: Ord>(mut target: Vec<T>, extra: impl IntoIterator<Item = T>) -> Vec<T> {
    target.extend(extra);
    target.sort_unstable();
    target.dedup();
    target
}

pub struct OutputDemandRetention {
    pub position: NonZeroU64,
    pub pending_denial: Option<QueryObligationDenial>,
}

impl QueryObligationHistory {
    pub fn output_checkpoint_prerequisites(
        &self,
    ) -> Vec<(DurableCompositeCommitIdentity, String, Vec<u8>)> {
        let state = self.state();
        let Some(journal) = state.journal.as_ref() else {
            return Vec::new();
        };
        let output = ledger_inspect::output(&journal.ledger);
        output
            .demands
            .values()
            .filter(|row| !output.completed.contains(&row.acquire_position))
            .flat_map(|row| {
                row.checkpoint_prerequisites
                    .iter()
                    .map(|payload| (row.basis, row.branch.clone(), payload.clone()))
            })
            .collect()
    }
    pub fn note_output_demand_completion(&self, position: NonZeroU64) {
        self.state().pending_output_discharges.insert(position);
    }
    pub fn flush_output_discharges(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        loop {
            let position = self.state().pending_output_discharges.first().copied();
            let Some(position) = position else {
                return Ok(());
            };
            self.discharge_output_demand(request, position)?;
            self.state().pending_output_discharges.remove(&position);
        }
    }
    pub fn retain_output_demand(
        &self,
        request: &ExecutionRequest<'_, '_>,
        mut row: QueryOutputDemandObligation,
        lifecycle: u64,
    ) -> Result<Option<OutputDemandRetention>, QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(None);
        };
        self.flush_output_discharges(request)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let prior = {
            let mut state = self.state();
            if state
                .retired_branches
                .contains(&(row.branch.clone(), lifecycle))
            {
                return Err(QueryObligationDenial::ObligationSubjectMismatch);
            }
            let pos = state.journal.as_ref().and_then(|journal| {
                ledger_inspect::output(&journal.ledger)
                    .subjects
                    .get(&ledger_inspect::output_demand_subject(&row))
                    .copied()
            });
            if let Some(pos) = pos {
                match state.demand_lifecycles.get(&pos).copied() {
                    Some(prior_lc) if prior_lc > lifecycle => {
                        return Err(QueryObligationDenial::ObligationSubjectMismatch);
                    }
                    Some(prior_lc) if prior_lc < lifecycle => Some((pos, None)),
                    _ => {
                        state.demand_lifecycles.entry(pos).or_insert(lifecycle);
                        let existing = ledger_inspect::output(
                            &state.journal.as_ref().expect("admitted journal").ledger,
                        )
                        .demands[&pos]
                            .clone();
                        Some((pos, Some(existing)))
                    }
                }
            } else {
                None
            }
        };
        if let Some((prior, existing)) = prior {
            if let Some(existing) = existing {
                let record = QueryOutputDemandRecord::Links {
                    acquire_position: prior,
                    performed_sources: merge_sorted(
                        existing.performed_sources,
                        row.performed_sources,
                    ),
                    prerequisites: merge_sorted(existing.prerequisites, row.prerequisites),
                    checkpoint_prerequisites: merge_sorted(
                        existing.checkpoint_prerequisites,
                        row.checkpoint_prerequisites,
                    ),
                };
                let pending_denial = driver.publish(medium.as_ref(), record, request).err();
                return Ok(Some(OutputDemandRetention {
                    position: prior,
                    pending_denial,
                }));
            }
            let discharge = QueryOutputDemandRecord::Discharge {
                acquire_position: prior,
            };
            driver.publish(medium.as_ref(), discharge, request)?;
        }
        let position = {
            let mut state = self.state();
            let position = state
                .journal
                .as_ref()
                .ok_or(QueryObligationDenial::SupportUnavailable)?
                .ledger
                .position()
                .checked_add(1)
                .and_then(NonZeroU64::new)
                .ok_or(QueryObligationDenial::IdentityExhausted)?;
            row.acquire_position = position;
            state
                .journal
                .as_mut()
                .expect("admitted journal")
                .prepare_pending(QueryOutputDemandRecord::Retain(row))?;
            state.demand_lifecycles.insert(position, lifecycle);
            position
        };
        let pending_denial = driver.finish_pending(medium.as_ref(), request).err();
        if pending_denial == Some(QueryObligationDenial::PublicationNoEffect) {
            return Err(QueryObligationDenial::PublicationNoEffect);
        }
        Ok(Some(OutputDemandRetention {
            position,
            pending_denial,
        }))
    }
    pub fn active_output_demands(
        &self,
    ) -> Vec<(NonZeroU64, DurableCompositeCommitIdentity, String)> {
        let state = self.state();
        let Some(journal) = state.journal.as_ref() else {
            return Vec::new();
        };
        let output = ledger_inspect::output(&journal.ledger);
        output
            .demands
            .values()
            .filter(|row| !output.completed.contains(&row.acquire_position))
            .map(|row| (row.acquire_position, row.basis, row.branch.clone()))
            .collect()
    }
    pub fn active_output_demands_for_branch(
        &self,
        branch: &str,
        lifecycle: u64,
    ) -> Vec<NonZeroU64> {
        let state = self.state();
        let Some(journal) = state.journal.as_ref() else {
            return Vec::new();
        };
        let output = ledger_inspect::output(&journal.ledger);
        output
            .demands
            .values()
            .filter(|row| {
                !output.completed.contains(&row.acquire_position)
                    && row.branch == branch
                    && state.demand_lifecycles.get(&row.acquire_position) == Some(&lifecycle)
            })
            .map(|row| row.acquire_position)
            .collect()
    }
    pub fn active_output_basis_pins(&self) -> Vec<(DurableCompositeCommitIdentity, String)> {
        let state = self.state();
        let Some(journal) = state.journal.as_ref() else {
            return Vec::new();
        };
        let output = ledger_inspect::output(&journal.ledger);
        let mut pins = Vec::new();
        for row in output.demands.values() {
            if !output.completed.contains(&row.acquire_position) {
                pins.push((row.basis, row.branch.clone()));
                for source in &row.performed_sources {
                    pins.push((*source, row.branch.clone()));
                }
            }
        }
        for row in output.sources.values() {
            pins.push((row.commit, row.branch.clone()));
        }
        pins
    }
    pub fn has_active_output_basis(&self, commit: DurableCompositeCommitIdentity) -> bool {
        let state = self.state();
        let Some(journal) = state.journal.as_ref() else {
            return false;
        };
        if let Some(frame) = journal.pending.as_ref() {
            match frame.record() {
                QueryObligationRecord::OutputDemand(QueryOutputDemandRecord::Retain(row))
                    if row.basis == commit || row.performed_sources.contains(&commit) =>
                {
                    return true;
                }
                QueryObligationRecord::OutputDemand(QueryOutputDemandRecord::Links {
                    performed_sources,
                    ..
                }) if performed_sources.contains(&commit) => {
                    return true;
                }
                QueryObligationRecord::OutputSource(QueryOutputSourceRecord::Retain(row))
                    if row.commit == commit =>
                {
                    return true;
                }
                _ => {}
            }
        }
        let output = ledger_inspect::output(&journal.ledger);
        output.sources.contains_key(&commit)
            || output.demands.values().any(|row| {
                !output.completed.contains(&row.acquire_position)
                    && (row.basis == commit || row.performed_sources.contains(&commit))
            })
    }
    pub fn has_output_demands(&self) -> bool {
        let state = self.state();
        state.journal.as_ref().is_some_and(|journal| {
            ledger_inspect::output(&journal.ledger)
                .subjects
                .values()
                .any(|position| !state.pending_output_discharges.contains(position))
        })
    }
    pub fn matching_output_demand(
        &self,
        row: &QueryOutputDemandObligation,
    ) -> Option<QueryOutputDemandObligation> {
        let state = self.state();
        let output = ledger_inspect::output(&state.journal.as_ref()?.ledger);
        let position = output
            .subjects
            .get(&ledger_inspect::output_demand_subject(row))?;
        if state.pending_output_discharges.contains(position) {
            return None;
        }
        output.demands.get(position).cloned()
    }
    pub fn output_demand(&self, position: NonZeroU64) -> Option<QueryOutputDemandObligation> {
        ledger_inspect::output(&self.state().journal.as_ref()?.ledger)
            .demands
            .get(&position)
            .cloned()
    }
    pub fn output_demand_retained(&self, position: NonZeroU64) -> bool {
        let state = self.state();
        !state.pending_output_discharges.contains(&position)
            && state.journal.as_ref().is_some_and(|journal| {
                let output = ledger_inspect::output(&journal.ledger);
                output.demands.contains_key(&position) && !output.completed.contains(&position)
            })
    }
    pub fn link_output_demand(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: NonZeroU64,
        lifecycle: u64,
        sources: Vec<DurableCompositeCommitIdentity>,
        prerequisites: Vec<NonZeroU64>,
        checkpoint_prerequisites: Vec<Vec<u8>>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let original = {
            let state = self.state();
            let row = state
                .journal
                .as_ref()
                .and_then(|journal| {
                    ledger_inspect::output(&journal.ledger)
                        .demands
                        .get(&position)
                })
                .cloned()
                .ok_or(QueryObligationDenial::UnknownObligation)?;
            if state
                .retired_branches
                .contains(&(row.branch.clone(), lifecycle))
                || state
                    .demand_lifecycles
                    .get(&position)
                    .is_some_and(|recorded| *recorded != lifecycle)
            {
                return Err(QueryObligationDenial::ObligationSubjectMismatch);
            }
            row
        };
        let sources = merge_sorted(sources, original.performed_sources.iter().copied());
        let prerequisites = merge_sorted(prerequisites, original.prerequisites.iter().copied());
        let checkpoint_prerequisites = merge_sorted(
            checkpoint_prerequisites,
            original.checkpoint_prerequisites.iter().cloned(),
        );
        if checkpoint_prerequisites == original.checkpoint_prerequisites
            && sources == original.performed_sources
            && prerequisites == original.prerequisites
        {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputDemandRecord::Links {
                acquire_position: position,
                performed_sources: sources,
                prerequisites,
                checkpoint_prerequisites,
            },
            request,
        )
    }
    pub fn discharge_output_demand(
        &self,
        request: &ExecutionRequest<'_, '_>,
        acquire_position: NonZeroU64,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if !self.state().journal.as_ref().is_some_and(|journal| {
            let output = ledger_inspect::output(&journal.ledger);
            output.demands.contains_key(&acquire_position)
                && !output.completed.contains(&acquire_position)
        }) {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputDemandRecord::Discharge { acquire_position },
            request,
        )
    }
    pub fn retain_output_domain_failure(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: NonZeroU64,
        failure: Vec<u8>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if self.output_domain_failure(position).as_ref() == Some(&failure) {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputDemandRecord::DomainDenied {
                acquire_position: position,
                failure,
            },
            request,
        )
    }
    pub fn output_domain_failure(&self, position: NonZeroU64) -> Option<Vec<u8>> {
        ledger_inspect::output(&self.state().journal.as_ref()?.ledger)
            .failures
            .get(&position)
            .cloned()
    }
}
