//! The consuming cursor is the only door to prepared producer application.
use super::*;
use worth_proof::CanonicalUniqueVec;

use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct CanonicalApplyCursor<
    'owner,
    Key,
    Value,
    Stop,
> {
    basis: Option<&'owner worth_relational::facade::runtime::PositionedRelationalSnapshot>,
    keys: CanonicalUniqueVec<Key>,
    slots: Vec<(
        OwnerPlan<'owner, Value, Stop>,
        Result<
            crate::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::ComputedMember,
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
        >,
    )>,
}

impl<'owner, Key, Value, Stop> CanonicalApplyCursor<'owner, Key, Value, Stop> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn compute_admitted(
        wave: crate::domain_computation::primary_graph::application_output_demand::AdmittedRequiredWave<'owner, Key, OwnerPlan<'owner, Value, Stop>>,
        execution: &QueryRequestExecution<'_>,
    ) -> Self {
        let (basis, keys, plans) = wave.into_parts();
        let mut cursor = Self::compute(keys, plans, execution);
        cursor.basis = Some(basis);
        cursor
    }

    fn compute(
        keys: CanonicalUniqueVec<Key>,
        mut plans: Vec<OwnerPlan<'owner, Value, Stop>>,
        execution: &QueryRequestExecution<'_>,
    ) -> Self {
        assert_eq!(
            keys.as_slice().len(),
            plans.len(),
            "one aligned slot per admitted member"
        );
        let members = plans
            .iter_mut()
            .map(
                |plan| super::super::super::super::computation::wave_dispatch::PreparedMember {
                    requests: std::mem::take(&mut plan.requests),
                },
            )
            .collect();
        let computed =
            super::super::super::super::computation::wave_dispatch::dispatch(execution, members);
        let slots = plans.into_iter().zip(computed).collect();
        Self {
            basis: None,
            keys,
            slots,
        }
    }

    /// The owner classifies each completion: holding a queued failure continues
    /// the cursor. A caller failure or request stop drops the inaccessible suffix.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn apply(
        self,
        admission: &mut InvalidationEditAdmission,
        mut published: impl FnMut(
            Key,
            Result<Value, Stop>,
            &mut InvalidationEditAdmission,
        ) -> Result<(), Stop>,
        refused: impl FnOnce(
            Key,
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
        ) -> Stop,
    ) -> Result<(), Stop> {
        let Self {
            basis: _basis,
            keys,
            slots,
        } = self;
        for (key, (plan, computed)) in keys.into_keys().into_iter().zip(slots) {
            let computed = match computed {
                Ok(computed) => computed,
                Err(denial) => return Err(refused(key, denial)),
            };
            let outcome = computed.finish(|slots| (plan.complete)(slots, admission));
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
    let cursor = CanonicalApplyCursor::compute(keys, vec![plan], execution);
    let mut outcome = None;
    cursor.apply(
        admission,
        |(), applied, _| {
            outcome = Some(applied?);
            Ok(())
        },
        |(), denial| {
            ProducerExecutionStop::ExecutionStopped(
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::ExecutionRequest(denial),
                    format!("member compute refused: {denial}"),
                ),
            )
        },
    )?;
    Ok(outcome.expect("the singleton cursor applied its one slot"))
}

#[cfg(test)]
mod tests;
