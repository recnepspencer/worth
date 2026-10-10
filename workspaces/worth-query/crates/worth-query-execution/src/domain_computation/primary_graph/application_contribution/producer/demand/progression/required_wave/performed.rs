//! One public advance retains each published member's exact decision input.
//!
//! Native publication coordinates never confer another turn. A new Fresh
//! permission exists only after a changed source meaning or changed decision
//! facts/consumed output evidence, compared by their existing owners.
use super::super::super::super::execution::ProducerExecutionStop;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryObservedSourceEpoch, output_lineage::AcceptedCurrentCandidate,
};

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PerformedMembers
{
    entries: Vec<PerformedMember>,
}
struct PerformedMember {
    key: WorthQueryOutputDemandKey,
    source: WorthQueryObservedSourceEpoch,
    input: Option<AcceptedCurrentCandidate>,
    performed: bool,
}

/// Only this owner can mint permission to enter the authoritative Fresh path.
/// Private fields make constructing an unchecked permission outside this
/// module impossible, including from NeedsDisclosure or a rejoined Ready.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct FreshReadiness
{
    member: usize,
    _changed_input: Option<ChangedDecisionInput>,
}

struct ChangedDecisionInput(());
impl FreshReadiness {
    fn changed(member: usize, evidence: ChangedDecisionInput) -> Self {
        Self {
            member,
            _changed_input: Some(evidence),
        }
    }
}
mod ordinary;
mod publication;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use ordinary::DecisionInput;
#[cfg(test)]
mod tests;

impl PerformedMembers {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn start(
    ) -> Self {
        Self {
            entries: Vec::new(),
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
                .charge_external_work(entry.comparison_work()?)
                .map_err(admission_denial)?;
            if entry.names(ready.key()) && entry.performed && entry.input.is_none() {
                admission
                    .charge_external_work(std::mem::size_of::<AcceptedCurrentCandidate>() as u64)
                    .map_err(admission_denial)?;
                entry.input = candidate.cloned();
            }
        }
        Ok(())
    }

    /// The permission identifies the exact record that admitted this attempt.
    /// Publication seals it before any checkpoint delivery or Ready rejoin.
    fn performed(&mut self, permission: FreshReadiness) {
        self.entries[permission.member].performed = true;
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn fresh<
        Schema: ApplicationSchema + 'static,
    >(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        source: WorthQueryObservedSourceEpoch,
        runtime: &DecisionInput<'_, '_, '_, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<FreshReadiness>, ProducerExecutionStop> {
        for (index, entry) in self.entries.iter_mut().enumerate() {
            admission
                .charge_external_work(entry.comparison_work()?)
                .map_err(admission_denial)?;
            if !entry.names(key) {
                continue;
            }
            if !entry.performed {
                admission
                    .charge_external_work(4)
                    .map_err(admission_denial)?;
                entry.source = source;
                return Ok(Some(FreshReadiness {
                    member: index,
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
            entry.source = source;
            entry.input = None;
            entry.performed = false;
            return Ok(Some(FreshReadiness::changed(
                index,
                ChangedDecisionInput(()),
            )));
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
            if replacement.capacity() < next {
                return Err(capacity_denial().into());
            }
            replacement.append(&mut self.entries);
            self.entries = replacement;
        }
        let index = self.entries.len();
        let key_bytes = key.producer_identity().len();
        admission
            .charge_external_work(key_bytes as u64)
            .map_err(admission_denial)?;
        admission
            .admit_read_scratch(key_bytes as u64)
            .map_err(admission_denial)?;
        self.entries.push(PerformedMember {
            key: key.clone(),
            source,
            input: None,
            performed: false,
        });
        Ok(Some(FreshReadiness {
            member: index,
            _changed_input: None,
        }))
    }
}

impl PerformedMember {
    fn comparison_work(&self) -> Result<u64, WorthQueryOutputDemandDenial> {
        self.key
            .producer_identity()
            .len()
            .checked_add(self.key.applicability().profile_kind().len())
            .and_then(|bytes| bytes.checked_add(3))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(work_denial)
    }
    fn names(&self, key: &WorthQueryOutputDemandKey) -> bool {
        self.key.family_type() == key.family_type()
            && self.key.producer_identity() == key.producer_identity()
            && self.key.applicability() == key.applicability()
            && self.key.source_epoch().same_occurrence(key.source_epoch())
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
