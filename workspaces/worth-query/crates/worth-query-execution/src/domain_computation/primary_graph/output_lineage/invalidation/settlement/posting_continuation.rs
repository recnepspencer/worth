//! Completed selected native rebase, paired with its exact normalized body.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;

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
        control: StorageControl<'_, '_>,
    ) -> Result<PostingContinuation, StoreDenial> {
        control.check_live()?;
        control.check_live()?;
        Ok(PostingContinuation(Arc::new(CompletedPostingRebase {
            basis: self,
            body,
            after,
        })))
    }
}
#[derive(Debug)]
struct CompletedPostingRebase {
    basis: PostingRebaseBasis,
    body: FactPayload,
    after: PositionedRelationalSnapshot,
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
    ) -> Result<Option<Arc<PostingProjection>>, CompanionPreflightStop> {
        let completed = &self.0;
        if completed.after != *selected || !completed.body.same_backing(facts.postconditions()) {
            return Ok(None);
        }

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

        let Some(binding) = state.posting_bindings.get(&projection.id) else {
            return Ok(None);
        };
        let Some(binding) = binding.settlements() else {
            return Ok(None);
        };

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
            if !same_addresses(old, new)? {
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
            if !same_addresses(old, new)? {
                return Ok(None);
            }
        }
        Ok(Some(Arc::clone(projection)))
    }
}

fn same_addresses(
    old: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    new: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
) -> Result<bool, CompanionPreflightStop> {
    let Some(old) = addresses(old)? else {
        return Ok(false);
    };
    let Some(new) = addresses(new)? else {
        return Ok(false);
    };
    if old.len() != new.len() {
        return Ok(false);
    }
    for (old, new) in old.iter().zip(new.iter()) {
        if old != new {
            return Ok(false);
        }
    }
    Ok(true)
}
fn addresses(
    fact: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
) -> Result<Option<OrdSet<Arc<FactPostingKey>>>, CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_fact();
    let mut keys = OrdSet::new();
    let result = super::super::fact_keys::visit(fact, |key| {
        keys.insert(Arc::new(key));
        Ok(())
    });
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
