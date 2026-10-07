//! Per-producer semantic output material before epoch-wide cause folding.
use super::{
    ApplyCommitPacket, ComparatorPolicyResolver, EvaluationWork, OutputCommitPreparationSeam,
    ProducedAspectDelta, SignalError, SignalGraph,
};
use crate::data::aspect::{Aspect, AspectMask, MAX_ASPECTS};
use crate::data::output::{ChangedRegion, PartitionSubscription};
use crate::data::proof::invalidation::output_commit::ProducedAspectChange;
use crate::data::request_preparation::SignalPreparationBudget;
use worth_execution::MapKernelContext;

pub(super) struct OutputCommitHead {
    pub(super) apply: ApplyCommitPacket,
    pub(super) artifact_write: Option<crate::data::trace::HotArtifactWrite>,
    pub(super) produced_delta: Option<ProducedAspectDelta>,
}

impl SignalGraph {
    pub(super) fn claim_epoch_delta_copy(
        &self,
        delta: &ProducedAspectDelta,
        mut preparation: Option<&mut SignalPreparationBudget>,
        request_work: Option<&mut MapKernelContext<'_, '_>>,
    ) -> Result<(), SignalError> {
        let mut visits = delta.changes.as_slice().len();
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<ProducedAspectChange>(delta.changes.as_slice().len())?;
        }
        for change in delta.changes.as_slice() {
            if !change.changed_scopes.is_empty() {
                if let Some(budget) = preparation.as_deref_mut() {
                    budget.claim_vec::<PartitionSubscription>(change.changed_scopes.len())?;
                }
            }
            for scope in change.changed_scopes.as_slice() {
                let depth = scope.path().depth();
                let bytes = scope
                    .path()
                    .checked_segment_bytes()
                    .ok_or_else(|| SignalError::invalid_input("delta scope copy size overflow"))?;
                visits = visits
                    .checked_add(depth)
                    .and_then(|n| n.checked_add(bytes))
                    .ok_or_else(|| SignalError::invalid_input("delta copy work overflow"))?;
                if let Some(budget) = preparation.as_deref_mut() {
                    budget.claim_vec::<String>(depth)?;
                    budget.claim_vec::<u8>(bytes)?;
                }
            }
        }
        if let Some(request) = request_work {
            request
                .checkpoint(
                    u64::try_from(visits)
                        .map_err(|_| SignalError::invalid_input("delta copy work overflow"))?,
                )
                .map_err(|_| SignalError::invalid_input("delta copy work stopped"))?;
        }
        Ok(())
    }

    pub(super) fn claim_epoch_output_head_shape(
        &self,
        apply: &ApplyCommitPacket,
        budget: &mut SignalPreparationBudget,
    ) -> Result<(), SignalError> {
        let effect = &apply.effect;
        let legacy = effect.changed_regions();
        // Runtime artifact scopes are copied once for the hot record and at
        // most once again for a retained cold intent.
        budget.claim_vec::<PartitionSubscription>(legacy.len())?;
        budget.claim_vec::<ChangedRegion>(legacy.len())?;
        for region in legacy {
            claim_region_path(region, budget)?;
            claim_region_path(region, budget)?;
        }
        if !matches!(
            effect.operational.verdict,
            crate::logic::evaluation::EvaluationVerdict::Recomputed
        ) {
            return Ok(());
        }
        // prepare_produced_delta_at_ordinal uses this fixed capacity even
        // when only a subset of aspects eventually emits a change.
        budget.claim_vec::<ProducedAspectChange>(MAX_ASPECTS)?;
        let previous = self.node_aspect_version(effect.operational.node)?;
        let produces = self
            .get_contract(effect.operational.node)?
            .semantics
            .produces;
        for (index, (&before, &after)) in previous
            .slots()
            .iter()
            .zip(effect.operational.aspect_version.slots())
            .enumerate()
        {
            let aspect = Aspect::new(index as u8);
            if before == after || !produces.contains(AspectMask::from_aspect(aspect)) {
                continue;
            }
            let exact_count = effect
                .changed_aspect_regions()
                .iter()
                .filter(|(owner, _)| *owner == aspect)
                .count();
            if exact_count == 0 {
                budget.claim_vec::<PartitionSubscription>(legacy.len())?;
                for region in legacy {
                    claim_region_path(region, budget)?;
                }
            } else {
                budget.claim_vec::<PartitionSubscription>(exact_count)?;
                for (_, region) in effect
                    .changed_aspect_regions()
                    .iter()
                    .filter(|(owner, _)| *owner == aspect)
                {
                    claim_region_path(region, budget)?;
                }
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn fail_second_output_preparation_for_test(&self) {
        use std::sync::atomic::Ordering;
        self.epoch_output_preparation_fault_after
            .store(2, Ordering::SeqCst);
    }

    #[cfg(test)]
    pub(super) fn check_epoch_output_preparation_fault(&self) -> Result<(), SignalError> {
        use std::sync::atomic::Ordering;
        let previous = self.epoch_output_preparation_fault_after.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |remaining| {
                (remaining > 0 && remaining < (1 << (usize::BITS - 1))).then(|| remaining - 1)
            },
        );
        if matches!(previous, Ok(1)) {
            return Err(SignalError::internal(
                "injected second output preparation failure",
            ));
        }
        Ok(())
    }

    pub(super) fn prepare_output_commit_head(
        &self,
        apply: ApplyCommitPacket,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        probe: &mut impl FnMut(OutputCommitPreparationSeam) -> Result<(), SignalError>,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<OutputCommitHead, SignalError> {
        self.prepare_output_commit_head_at_ordinal(
            apply,
            comparator_resolver,
            self.cause_sets.reserve_output_commit_ordinal(),
            probe,
            work,
        )
    }

    pub(super) fn prepare_output_commit_head_at_ordinal(
        &self,
        mut apply: ApplyCommitPacket,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        ordinal: crate::data::proof::invalidation::binding::OutputCommitOrdinal,
        probe: &mut impl FnMut(OutputCommitPreparationSeam) -> Result<(), SignalError>,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<OutputCommitHead, SignalError> {
        self.apply_semantic_output_commit_decision(&mut apply, comparator_resolver, work)?;
        let artifact_write = self.build_semantic_artifact_write(&mut apply, work)?;
        probe(OutputCommitPreparationSeam::SemanticDecision)?;
        let produced_delta = self.prepare_produced_delta_at_ordinal(&apply, ordinal, work)?;
        probe(OutputCommitPreparationSeam::ProducedDelta)?;
        Ok(OutputCommitHead {
            apply,
            artifact_write,
            produced_delta,
        })
    }
}

fn claim_region_path(
    region: &ChangedRegion,
    budget: &mut SignalPreparationBudget,
) -> Result<(), SignalError> {
    budget.claim_vec::<String>(region.path().depth())?;
    budget.claim_vec::<u8>(region.path().total_segment_bytes())
}
