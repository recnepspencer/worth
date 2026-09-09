//! Semantic output comparison and its artifact/delta decision.
use super::{
    ApplyCommitPacket, Aspect, AspectMask, ComparatorPolicyResolver, EvaluationVerdict,
    SignalError, SignalGraph, SuppressionReason,
};

impl SignalGraph {
    pub(super) fn build_semantic_artifact_write(
        &self,
        apply: &mut ApplyCommitPacket,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<Option<crate::data::trace::HotArtifactWrite>, SignalError> {
        let (previous_output, previous_continuity) =
            self.node_runtime_artifact_output_tokens(apply.effect.operational.node)?;
        let mut comparison = self.compare_effect(
            &apply.effect,
            previous_output,
            previous_continuity,
            self.node_eval_config(apply.effect.operational.node)?
                .output_equivalence
                .clone(),
            work,
        )?;
        if matches!(
            apply.effect.operational.verdict,
            EvaluationVerdict::Suppressed {
                reason: SuppressionReason::ComparatorMatch
                    | SuppressionReason::OutputIdentityUnchanged
                    | SuppressionReason::ContinuityTokenUnchanged,
            }
        ) {
            comparison.propagation_suppressed = true;
        }
        apply.comparison = comparison;
        self.build_effect_artifact_write(
            &apply.effect,
            previous_output,
            previous_continuity,
            apply.comparison,
            work,
        )
    }

    pub(super) fn apply_semantic_output_commit_decision(
        &self,
        apply: &mut ApplyCommitPacket,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(4 * crate::data::aspect::MAX_ASPECTS))?;
        let node = apply.effect.operational.node;
        let previous = self.node_aspect_version(node)?;
        if matches!(
            apply.effect.operational.verdict,
            EvaluationVerdict::Suppressed {
                reason: SuppressionReason::ComparatorMatch
                    | SuppressionReason::OutputIdentityUnchanged
                    | SuppressionReason::ContinuityTokenUnchanged,
            }
        ) {
            apply.effect.operational.aspect_version = previous;
            apply.effect.operational.output_change = crate::data::output::OutputChange::Unchanged;
            return Ok(());
        }
        if !matches!(
            apply.effect.operational.verdict,
            EvaluationVerdict::Recomputed
        ) {
            return Ok(());
        }
        if !self.node_runtime_artifact_state_present(node)?
            && apply.effect.operational.aspect_version != previous
        {
            return Ok(());
        }
        if apply.comparison.propagation_suppressed {
            apply.effect.operational.aspect_version = previous;
            apply.effect.operational.output_change = crate::data::output::OutputChange::Unchanged;
            apply.effect.operational.verdict = EvaluationVerdict::Suppressed {
                reason: SuppressionReason::OutputIdentityUnchanged,
            };
            return Ok(());
        }
        let candidate = apply.effect.operational.aspect_version;
        let config = self.node_eval_config(node)?;
        let produces = self.get_contract(node)?.semantics.produces;
        let mut committed = previous;
        for (index, (&cached, &current)) in
            previous.slots().iter().zip(candidate.slots()).enumerate()
        {
            let aspect = Aspect::new(index as u8);
            if produces.contains(AspectMask::from_aspect(aspect))
                && config.output_equivalence.has_meaningful_change(
                    aspect,
                    cached,
                    current,
                    comparator_resolver,
                )?
            {
                committed = committed.with(aspect, current);
            }
        }
        apply.effect.operational.aspect_version = committed;
        if committed == previous {
            apply.effect.operational.output_change = crate::data::output::OutputChange::Unchanged;
            apply.effect.operational.verdict = EvaluationVerdict::Suppressed {
                reason: SuppressionReason::ComparatorMatch,
            };
        }
        Ok(())
    }
}
