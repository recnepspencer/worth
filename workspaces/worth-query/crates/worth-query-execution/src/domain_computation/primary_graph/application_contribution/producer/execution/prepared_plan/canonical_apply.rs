//! The consuming cursor is the only door to prepared producer application.
use super::*;
use worth_proof::CanonicalUniqueVec;

use crate::domain_computation::primary_graph::application_contribution::{
    computation::prepared_request::ComputedSlot, QueryRequestExecution, WorthQueryMemberExecution,
};

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct CanonicalApplyCursor<
    'owner,
    Key,
    Value,
    Stop,
> {
    keys: CanonicalUniqueVec<Key>,
    slots: Vec<(OwnerPlan<'owner, Value, Stop>, Vec<ComputedSlot>)>,
}

impl<'owner, Key, Value, Stop> CanonicalApplyCursor<'owner, Key, Value, Stop> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn compute_serial(
        keys: CanonicalUniqueVec<Key>,
        plans: Vec<OwnerPlan<'owner, Value, Stop>>,
        execution: &QueryRequestExecution<'_>,
    ) -> Self {
        assert_eq!(
            keys.as_slice().len(),
            plans.len(),
            "one aligned slot per admitted member"
        );
        let slots = plans
            .into_iter()
            .map(|mut plan| {
                let computed = std::mem::take(&mut plan.requests)
                    .into_iter()
                    .map(|request| request.run(WorthQueryMemberExecution::new(execution)))
                    .collect();
                (plan, computed)
            })
            .collect();
        Self { keys, slots }
    }

    /// Consumes all slots. A failure drops the suffix, whose values have no
    /// accessor and whose completion callbacks can no longer apply anything.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn apply(
        self,
        admission: &mut InvalidationEditAdmission,
        mut published: impl FnMut(Key, Value, &mut InvalidationEditAdmission) -> Result<(), Stop>,
    ) -> Result<(), Stop> {
        let Self { keys, slots } = self;
        for (key, (plan, computed)) in keys.into_keys().into_iter().zip(slots) {
            let outcome = (plan.complete)(computed, admission)?;
            published(key, outcome, admission)?;
        }
        Ok(())
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn run_one(
    plan: ProducerExecutionPlan<'_>,
    execution: &QueryRequestExecution<'_>,
    admission: &mut InvalidationEditAdmission,
) -> Result<ProducerExecutionOutcome, ProducerExecutionStop> {
    let keys = CanonicalUniqueVec::try_from_sorted_unique(vec![()])
        .unwrap_or_else(|_| unreachable!("the singleton key is canonical"));
    let cursor = CanonicalApplyCursor::compute_serial(keys, vec![plan], execution);
    let mut outcome = None;
    cursor.apply(admission, |(), applied, _| {
        outcome = Some(applied);
        Ok(())
    })?;
    Ok(outcome.expect("the singleton cursor applied its one slot"))
}

#[cfg(test)]
mod tests;
