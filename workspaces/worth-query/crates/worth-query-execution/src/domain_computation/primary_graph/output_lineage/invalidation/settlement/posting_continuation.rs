//! Completed selected native rebase, paired with its exact normalized body.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    application_checkpoint::facts::FactPayload,
};
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct PostingRebaseBasis {
    prior: Arc<RecordedSettlementIdentity>,
    baseline: FactPayload,
    storage: PostingComputationStorage,
    selected: OrdSet<usize>,
}
impl PostingRebaseBasis {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn from_carry(
        prior: Arc<RecordedSettlementIdentity>,
        baseline: FactPayload,
        storage: PostingComputationStorage,
        selected: OrdSet<usize>,
    ) -> Self {
        Self {
            prior,
            baseline,
            storage,
            selected,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        body: FactPayload,
        after: PositionedRelationalSnapshot,
        selection_memory: Option<ExecutionMemoryReservation>,
        control: StorageControl<'_, '_>,
    ) -> Result<PostingContinuation, StoreDenial> {
        control.check_live()?;
        let bytes = index_capacity::arc_bytes::<CompletedPostingRebase>()
            .and_then(|bytes| {
                bytes.checked_add(u64::try_from(after.branch_id().0.capacity()).ok()?)
            })
            .ok_or(StoreDenial::Representability)?;
        let memory = control.policy().reserve_memory(bytes);
        control.check_live()?;
        Ok(PostingContinuation(Arc::new(CompletedPostingRebase {
            basis: self,
            body,
            after,
            _selection_memory: selection_memory,
            _memory: memory?,
        })))
    }
}
#[derive(Debug)]
struct CompletedPostingRebase {
    basis: PostingRebaseBasis,
    body: FactPayload,
    after: PositionedRelationalSnapshot,
    _selection_memory: Option<ExecutionMemoryReservation>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct PostingContinuation(
    Arc<CompletedPostingRebase>,
);
impl PartialEq for PostingContinuation {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for PostingContinuation {}

impl PostingContinuation {
    /// Only changed native ordinals and the actual output witness are projected.
    /// This proves addresses, never source currentness or new read permission.
    pub(super) fn checked_projection(
        &self,
        state: &MarkState,
        facts: &super::super::super::RetainedSourceFacts,
        output: Option<&RegisteredOutputFacts>,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<PostingProjection>>, CompanionPreflightStop> {
        let completed = &self.0;
        if completed.after != *selected || !completed.body.same_backing(facts.postconditions()) {
            return Ok(None);
        }
        admission.ordered_read(state.settlements.len())?;
        let Some(prior) = state.settlements.get(&completed.basis.prior) else {
            return Ok(None);
        };
        let projection = &completed.basis.storage.projection;
        if prior.delivery_epoch != state.delivery_epoch
            || prior.verification_requirement.is_some()
            || !Arc::ptr_eq(&prior.posting_projection, projection)
            || !prior
                .facts
                .postconditions()
                .same_backing(&completed.basis.baseline)
            || projection.requirement.is_some()
            || projection.source_len != facts.len()
            || projection.output_len != output.map_or(0, |set| set.facts.len())
        {
            return Ok(None);
        }
        admission.ordered_read(state.posting_bindings.len())?;
        let Some(binding) = state.posting_bindings.get(&projection.id) else {
            return Ok(None);
        };
        let Some(binding) = binding.settlements() else {
            return Ok(None);
        };
        admission.ordered_read(binding.settlements.len())?;
        if !Arc::ptr_eq(&binding.projection, projection)
            || !binding.settlements.contains(&completed.basis.prior)
        {
            return Ok(None);
        }
        for &ordinal in &completed.basis.selected {
            let (Some(old), Some(new)) = (
                completed.basis.baseline.get(ordinal),
                completed.body.get(ordinal),
            ) else {
                return Ok(None);
            };
            if !same_addresses(old, new, admission)? {
                return Ok(None);
            }
        }
        let old_output = prior
            .output_facts
            .as_ref()
            .map(|set| set.facts.as_ref())
            .unwrap_or(&[]);
        let new_output = output.map(|set| set.facts.as_ref()).unwrap_or(&[]);
        if old_output.len() != new_output.len() {
            return Ok(None);
        }
        for (old, new) in old_output.iter().zip(new_output) {
            if !same_addresses(old, new, admission)? {
                return Ok(None);
            }
        }
        Ok(Some(Arc::clone(projection)))
    }
}

fn same_addresses(
    old: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    new: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, CompanionPreflightStop> {
    let Some(old) = addresses(old, admission)? else {
        return Ok(false);
    };
    let Some(new) = addresses(new, admission)? else {
        return Ok(false);
    };
    if old.len() != new.len() {
        return Ok(false);
    }
    for (old, new) in old.iter().zip(new.iter()) {
        admission.work(
            old.comparison_work_bound()
                .and_then(|work| work.checked_add(new.comparison_work_bound()?))
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        if old != new {
            return Ok(false);
        }
    }
    Ok(true)
}
fn addresses(
    fact: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<OrdSet<Arc<FactPostingKey>>>, CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_fact();
    let meter = std::cell::RefCell::new(admission);
    let mut keys = OrdSet::new();
    let result = super::super::fact_keys::visit(
        fact,
        |work, bytes| {
            let mut meter = meter.borrow_mut();
            meter.work(work as u64)?;
            meter.bytes(bytes as u64)
        },
        |key| {
            let mut meter = meter.borrow_mut();
            meter.fresh_key_insert::<Arc<FactPostingKey>, ()>(&key, keys.len())?;
            meter.bytes(
                index_capacity::arc_bytes::<FactPostingKey>()
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
            )?;
            keys.insert(Arc::new(key));
            Ok(())
        },
    );
    match result {
        Ok(()) => Ok(Some(keys)),
        Err(super::super::fact_keys::FactKeyProjectionStop::Admission(stop)) => Err(stop),
        Err(super::super::fact_keys::FactKeyProjectionStop::FullVerificationRequired(_)) => {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests;
