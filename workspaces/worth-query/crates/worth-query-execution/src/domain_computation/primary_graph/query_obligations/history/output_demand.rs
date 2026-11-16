//! Lifecycle records use the one serialized Query publisher.
use super::*;
use journal::*;
use std::num::NonZeroU64;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableCompositeCommitIdentity;

pub(in crate::domain_computation::primary_graph) struct OutputDemandRetention {
    pub position: NonZeroU64,
    pub pending_denial: Option<QueryObligationDenial>,
}

impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn output_checkpoint_prerequisites(
        &self,
    ) -> Vec<(DurableCompositeCommitIdentity, String, Vec<u8>)> {
        let state = self.state();
        let Some(journal) = state.journal.as_ref() else {
            return Vec::new();
        };
        journal
            .ledger
            .output
            .demands
            .values()
            .filter(|row| {
                !journal
                    .ledger
                    .output
                    .completed
                    .contains(&row.acquire_position)
            })
            .flat_map(|row| {
                row.checkpoint_prerequisites
                    .iter()
                    .map(|payload| (row.basis, row.branch.clone(), payload.clone()))
            })
            .collect()
    }
    pub(in crate::domain_computation::primary_graph) fn note_output_demand_completion(
        &self,
        position: NonZeroU64,
    ) {
        self.state().pending_output_discharges.insert(position);
    }
    pub(in crate::domain_computation::primary_graph) fn flush_output_discharges(
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
    pub(in crate::domain_computation::primary_graph) fn retain_output_demand(
        &self,
        request: &ExecutionRequest<'_, '_>,
        mut row: QueryOutputDemandObligation,
    ) -> Result<Option<OutputDemandRetention>, QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(None);
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let position = {
            let state = self.state();
            state
                .journal
                .as_ref()
                .ok_or(QueryObligationDenial::SupportUnavailable)?
                .ledger
                .position()
                .checked_add(1)
                .and_then(NonZeroU64::new)
                .ok_or(QueryObligationDenial::IdentityExhausted)?
        };
        row.acquire_position = position;
        let prior = {
            let state = self.state();
            state
                .journal
                .as_ref()
                .and_then(|journal| journal.ledger.output.subjects.get(&row.subject()).copied())
        };
        if let Some(prior) = prior {
            let existing = self
                .state()
                .journal
                .as_ref()
                .expect("admitted journal")
                .ledger
                .output
                .demands[&prior]
                .clone();
            let mut sources = existing.performed_sources;
            sources.extend(row.performed_sources);
            sources.sort_unstable();
            sources.dedup();
            let mut prerequisites = existing.prerequisites;
            prerequisites.extend(row.prerequisites);
            prerequisites.sort_unstable();
            prerequisites.dedup();
            let mut checkpoint_prerequisites = existing.checkpoint_prerequisites;
            checkpoint_prerequisites.extend(row.checkpoint_prerequisites);
            checkpoint_prerequisites.sort_unstable();
            checkpoint_prerequisites.dedup();
            let pending_denial = driver
                .publish(
                    medium.as_ref(),
                    QueryOutputDemandRecord::Links {
                        acquire_position: prior,
                        performed_sources: sources,
                        prerequisites,
                        checkpoint_prerequisites,
                    },
                    request,
                )
                .err();
            return Ok(Some(OutputDemandRetention {
                position: prior,
                pending_denial,
            }));
        }
        self.state()
            .journal
            .as_mut()
            .expect("admitted journal")
            .prepare_pending(QueryOutputDemandRecord::Retain(row))?;
        let pending_denial = driver.finish_pending(medium.as_ref(), request).err();
        if pending_denial == Some(QueryObligationDenial::PublicationNoEffect) {
            return Err(QueryObligationDenial::PublicationNoEffect);
        }
        Ok(Some(OutputDemandRetention {
            position,
            pending_denial,
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn has_output_demands(&self) -> bool {
        self.state()
            .journal
            .as_ref()
            .is_some_and(|journal| !journal.ledger.output.subjects.is_empty())
    }
    pub(in crate::domain_computation::primary_graph) fn matching_output_demand(
        &self,
        row: &QueryOutputDemandObligation,
    ) -> Option<QueryOutputDemandObligation> {
        let state = self.state();
        let output = &state.journal.as_ref()?.ledger.output;
        let position = output.subjects.get(&row.subject())?;
        output.demands.get(position).cloned()
    }
    pub(in crate::domain_computation::primary_graph) fn output_demand(
        &self,
        position: NonZeroU64,
    ) -> Option<QueryOutputDemandObligation> {
        self.state()
            .journal
            .as_ref()?
            .ledger
            .output
            .demands
            .get(&position)
            .cloned()
    }
    pub(in crate::domain_computation::primary_graph) fn output_demand_retained(
        &self,
        position: NonZeroU64,
    ) -> bool {
        self.state().journal.as_ref().is_some_and(|journal| {
            journal.ledger.output.demands.contains_key(&position)
                && !journal.ledger.output.completed.contains(&position)
        })
    }
    pub(in crate::domain_computation::primary_graph) fn link_output_demand(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: NonZeroU64,
        mut sources: Vec<DurableCompositeCommitIdentity>,
        mut prerequisites: Vec<NonZeroU64>,
        mut checkpoint_prerequisites: Vec<Vec<u8>>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let original = {
            let state = self.state();
            state
                .journal
                .as_ref()
                .and_then(|journal| journal.ledger.output.demands.get(&position))
                .cloned()
                .ok_or(QueryObligationDenial::UnknownObligation)?
        };
        sources.extend(original.performed_sources.iter().copied());
        prerequisites.extend(original.prerequisites.iter().copied());
        sources.sort_unstable();
        sources.dedup();
        prerequisites.sort_unstable();
        prerequisites.dedup();
        checkpoint_prerequisites.extend(original.checkpoint_prerequisites.iter().cloned());
        checkpoint_prerequisites.sort_unstable();
        checkpoint_prerequisites.dedup();
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
    pub(in crate::domain_computation::primary_graph) fn discharge_output_demand(
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
            journal
                .ledger
                .output
                .demands
                .contains_key(&acquire_position)
                && !journal.ledger.output.completed.contains(&acquire_position)
        }) {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputDemandRecord::Discharge { acquire_position },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn retain_output_domain_failure(
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
    pub(in crate::domain_computation::primary_graph) fn output_domain_failure(
        &self,
        position: NonZeroU64,
    ) -> Option<Vec<u8>> {
        self.state()
            .journal
            .as_ref()?
            .ledger
            .output
            .failures
            .get(&position)
            .cloned()
    }
}
