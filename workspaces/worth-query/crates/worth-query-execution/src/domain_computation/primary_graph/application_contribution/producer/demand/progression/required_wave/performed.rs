//! One advance retains every attempted member's exact decision input.
//!
//! Native publication coordinates never confer another turn. A new Fresh
//! permission exists only after a changed source meaning or changed decision
//! facts/consumed output evidence, compared by their existing owners.
use super::super::super::super::execution::ProducerExecutionStop;
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryObservedSourceEpoch, output_lineage::AcceptedCurrentCandidate,
};
use std::any::TypeId;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PerformedMembers
{
    entries: Vec<PerformedMember>,
    selected: Option<usize>,
}
struct PerformedMember {
    family: TypeId,
    source: WorthQueryObservedSourceEpoch,
    input: Option<AcceptedCurrentCandidate>,
    performed: bool,
}

/// Only this owner can mint permission to enter the authoritative Fresh path.
/// Private fields make constructing an unchecked permission outside this
/// module impossible, including from NeedsDisclosure or a rejoined Ready.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct FreshReadiness
{
    _changed_input: Option<ChangedDecisionInput>,
}

struct ChangedDecisionInput(());
impl FreshReadiness {
    fn changed(evidence: ChangedDecisionInput) -> Self {
        Self {
            _changed_input: Some(evidence),
        }
    }
}
#[cfg(test)]
mod tests;

impl PerformedMembers {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn start(
    ) -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
        }
    }

    /// The first exact accepted cell after a Fresh attempt seals its performed
    /// facts and consumed identities. The record survives every wave reselect.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn capture(
        &mut self,
        ready: &SelectedReadyReadmission,
        candidate: Option<&AcceptedCurrentCandidate>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        for entry in &mut self.entries {
            admission
                .charge_external_work(3)
                .map_err(admission_denial)?;
            if entry.family == ready.key().family_type()
                && entry.source.same_occurrence(ready.key().source_epoch())
                && entry.performed
                && entry.input.is_none()
            {
                admission
                    .charge_external_work(std::mem::size_of::<AcceptedCurrentCandidate>() as u64)
                    .map_err(admission_denial)?;
                entry.input = candidate.cloned();
            }
        }
        Ok(())
    }

    /// Seals only a Ready the just-executed Fresh successor actually published.
    /// Pre-effect refusals remain unperformed and may resolve prerequisites.
    pub(super) fn performed(
        &mut self,
        ready: &SelectedReadyReadmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let index = self.selected.take().ok_or_else(foreign_denial)?;
        let entry = &mut self.entries[index];
        if entry.family != ready.key().family_type()
            || !entry.source.same_occurrence(ready.key().source_epoch())
        {
            return Err(foreign_denial());
        }
        entry.performed = true;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn fresh<
        Schema: ApplicationSchema + 'static,
    >(
        &mut self,
        family: TypeId,
        source: WorthQueryObservedSourceEpoch,
        runtime: &SelectedDecisionInput<'_, '_, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<FreshReadiness>, ProducerExecutionStop> {
        for (index, entry) in self.entries.iter_mut().enumerate() {
            admission
                .charge_external_work(3)
                .map_err(admission_denial)?;
            if entry.family != family || !entry.source.same_occurrence(&source) {
                continue;
            }
            if !entry.performed {
                admission
                    .charge_external_work(4)
                    .map_err(admission_denial)?;
                self.selected = Some(index);
                entry.source = source;
                return Ok(Some(FreshReadiness {
                    _changed_input: None,
                }));
            }
            let different = !entry.source.same_semantic_source(&source)
                || match &entry.input {
                    Some(input) => runtime.changed(input, admission)?,
                    None => false,
                };
            if !different {
                return Ok(None);
            }
            admission
                .charge_external_work(4)
                .map_err(admission_denial)?;
            self.selected = Some(index);
            entry.source = source;
            entry.input = None;
            entry.performed = false;
            return Ok(Some(FreshReadiness::changed(ChangedDecisionInput(()))));
        }
        let item = std::mem::size_of::<PerformedMember>();
        admission
            .charge_external_work((item + 4) as u64)
            .map_err(admission_denial)?;
        if self.entries.len() == self.entries.capacity() {
            let next = self.entries.capacity().saturating_mul(2).max(1);
            let peak = next
                .checked_add(self.entries.capacity())
                .and_then(|count| count.checked_mul(item))
                .ok_or_else(capacity_denial)?;
            admission
                .admit_read_scratch(u64::try_from(peak).map_err(|_| capacity_denial())?)
                .map_err(admission_denial)?;
            let copy_work = self
                .entries
                .len()
                .checked_mul(item)
                .and_then(|bytes| bytes.checked_mul(2))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(u64::try_from(copy_work).map_err(|_| work_denial())?)
                .map_err(admission_denial)?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(next)
                .map_err(|_| capacity_denial())?;
            if replacement.capacity() != next {
                return Err(capacity_denial().into());
            }
            replacement.append(&mut self.entries);
            self.entries = replacement;
        }
        self.selected = Some(self.entries.len());
        self.entries.push(PerformedMember {
            family,
            source,
            input: None,
            performed: false,
        });
        Ok(Some(FreshReadiness {
            _changed_input: None,
        }))
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct SelectedDecisionInput<
    'a,
    'runtime,
    Schema,
> {
    pub shared: &'a SharedSelectedProductOperation<'runtime, Schema>,
    pub positioned: &'a PositionedRelationalSnapshot,
    pub runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
}
impl<Schema: ApplicationSchema + 'static> SelectedDecisionInput<'_, '_, Schema> {
    fn changed(
        &self,
        input: &AcceptedCurrentCandidate,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, ProducerExecutionStop> {
        let graph = &self.runtime.primary_provider.graph;
        graph
            .with_runtime(|runtime| {
                input.decision_input_changed_at(
                    &graph.source_owner.invalidation_owner,
                    runtime,
                    self.shared.selected().application_basis().snapshot_handle(),
                    self.positioned,
                    admission,
                )
            })
            .map_err(|stop| {
                super::super::super::super::execution::ready_currentness_denial("", stop)
            })
    }
}
