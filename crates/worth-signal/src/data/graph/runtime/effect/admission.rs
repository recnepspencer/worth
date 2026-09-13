use crate::data::error::SignalError;
use crate::data::output::{ArtifactContinuityToken, OutputIdentity};
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::trace::{
    ArtifactMergeAuthority, ArtifactTransitionKey, CompactChangedScopeProof,
    ContinuityAuthorityToken, HotArtifactWrite, ReuseOperationalBasis, RuntimeArtifactHot,
    RuntimeArtifactState, RuntimeArtifactWarm,
};
use crate::logic::evaluation::EvaluationWork;
use crate::logic::evaluation::{
    EffectComparison, EvaluationEffect, EvaluationVerdict, SuppressionReason,
};

use super::vocabulary::{
    count_changed_partitions, normalize_output_change, trace_identity_hash, trace_output_hash,
    verdict_retains_runtime_artifact,
};
use super::{
    artifact_payload_work, cold_artifact::build_cold_artifact_intent,
    region_preparation::copy_region_scopes,
};
use super::{ApplyCommitPacket, SignalGraph};

impl SignalGraph {
    pub(super) fn compare_effect(
        &self,
        effect: &EvaluationEffect,
        previous_output: Option<&OutputIdentity>,
        previous_continuity: Option<&ArtifactContinuityToken>,
        output_equivalence: OutputEquivalencePolicy,
        work: &mut EvaluationWork<'_>,
    ) -> Result<EffectComparison, SignalError> {
        // The contract decision walks and normalizes only the fixed aspect
        // domain (8 or 32 slots), never a provider-sized collection.
        let aspects = crate::data::aspect::MAX_ASPECTS;
        work.reserve(Some(8 * aspects * aspects + 8 * aspects))?;
        for token in [
            effect.output_identity().map(OutputIdentity::as_str),
            effect
                .continuity_token()
                .map(ArtifactContinuityToken::as_str),
        ]
        .into_iter()
        .flatten()
        {
            work.reserve(token.len().checked_mul(2).and_then(|n| n.checked_add(1)))?;
        }
        crate::data::proof::invalidation::output_commit::SemanticOutputCommitDecision::validate_declared_change(
            self.node_aspect_version(effect.operational.node)?,
            effect.operational.aspect_version,
            self.get_contract(effect.operational.node)?.semantics.produces,
            effect.operational.output_change == crate::data::output::OutputChange::Unchanged,
        )
        .map_err(|violation| {
            SignalError::invalid_input(format!("output commit contract violation: {violation:?}"))
        })?;
        let output_identity_unchanged = matches!(
            (
                previous_output,
                effect.output_identity()
            ),
            (Some(previous), Some(current)) if previous == current
        );
        let continuity_token_unchanged = matches!(
            (
                previous_continuity,
                effect.continuity_token()
            ),
            (Some(previous), Some(current)) if previous == current
        );
        let propagation_suppressed =
            matches!(output_equivalence, OutputEquivalencePolicy::OutputIdentity)
                && output_identity_unchanged
                && !matches!(
                    effect.operational.verdict,
                    EvaluationVerdict::Deferred { .. }
                        | EvaluationVerdict::Suppressed {
                            reason: SuppressionReason::ValidatedClean
                                | SuppressionReason::ConditionRevertedClean,
                        }
                );

        Ok(EffectComparison {
            output_identity_unchanged,
            continuity_token_unchanged,
            propagation_suppressed,
            output_change: normalize_output_change(
                effect.operational.output_change,
                output_identity_unchanged,
                effect.output_identity().is_some(),
            ),
            changed_partition_count: count_changed_partitions(effect.changed_regions(), work)?,
        })
    }

    pub(super) fn build_effect_artifact_write(
        &self,
        effect: &EvaluationEffect,
        previous_output: Option<&OutputIdentity>,
        previous_continuity: Option<&ArtifactContinuityToken>,
        comparison: EffectComparison,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Option<HotArtifactWrite>, SignalError> {
        if !verdict_retains_runtime_artifact(&effect.operational.verdict) {
            return Ok(None);
        }
        let previous_hot = self.node_runtime_artifact_hot(effect.operational.node)?;
        let recomputed = matches!(effect.operational.verdict, EvaluationVerdict::Recomputed);
        let output_identity = if recomputed {
            effect.output_identity()
        } else {
            previous_output.or_else(|| effect.output_identity())
        };
        let continuity = if recomputed {
            effect.continuity_token()
        } else {
            previous_continuity.or_else(|| effect.continuity_token())
        };
        artifact_payload_work::warm(effect, work)?;
        artifact_payload_work::string(output_identity.map(OutputIdentity::as_str), work)?;
        artifact_payload_work::string(continuity.map(ArtifactContinuityToken::as_str), work)?;
        work.reserve(Some(crate::data::aspect::MAX_ASPECTS * 4))?;
        if recomputed {
            artifact_payload_work::string(
                effect.output_identity().map(OutputIdentity::as_str),
                work,
            )?;
        }
        let changed_scopes = copy_region_scopes(
            effect.changed_regions().iter(),
            effect.changed_regions().len(),
            work,
        )?;
        let cold_intent = build_cold_artifact_intent(
            effect,
            &self.installed_runtime_policy().retention_budget(),
            &changed_scopes,
            work,
        )?;
        let write = Some(HotArtifactWrite {
            runtime: Some(RuntimeArtifactState::new(
                RuntimeArtifactHot {
                    output_hash: if matches!(
                        effect.operational.verdict,
                        EvaluationVerdict::Recomputed
                    ) {
                        effect
                            .output_identity()
                            .map(trace_identity_hash)
                            .unwrap_or_else(|| trace_output_hash(effect.operational.aspect_version))
                    } else {
                        previous_hot
                            .map(|trace| trace.output_hash)
                            .unwrap_or_else(|| trace_output_hash(effect.operational.aspect_version))
                    },
                    output_change: comparison.output_change,
                    recomputed: effect.recomputed(),
                    dependency_count: effect.operational.dependency_snapshot_update.entry_count()
                        as u32,
                    meaningful_input_changes: effect.operational.meaningful_input_changes,
                    changed_partition_count: comparison.changed_partition_count,
                    propagation_suppressed: comparison.propagation_suppressed,
                    changed_scopes: CompactChangedScopeProof::new(changed_scopes),
                },
                RuntimeArtifactWarm {
                    output_identity: output_identity.cloned(),
                    continuity_token: ContinuityAuthorityToken::new(continuity.cloned()),
                    memoized_origin: effect.memoized_origin(),
                    reuse_basis: ReuseOperationalBasis::new(effect.operational.reuse_basis.clone()),
                    reuse_origin: effect.operational.reuse_origin,
                    reuse_boundary_authority: Some(
                        effect.operational.reuse_boundary_authority.clone(),
                    ),
                    lineage_artifact_id: ArtifactTransitionKey::default(),
                    merge_authority: ArtifactMergeAuthority::default(),
                },
            )),
            cold_intent,
        });
        Ok(write)
    }

    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    pub(crate) fn build_apply_commit_packet(
        &self,
        effect: EvaluationEffect,
        output_equivalence: OutputEquivalencePolicy,
        defer_snapshot_commit: bool,
        work: &mut EvaluationWork<'_>,
    ) -> Result<ApplyCommitPacket, SignalError> {
        let stored = if effect.previous_artifact_warm().is_none() {
            self.node_runtime_artifact_output_tokens(effect.operational.node)?
        } else {
            (None, None)
        };
        let (previous_output, previous_continuity) = match effect.previous_artifact_warm() {
            Some(snapshot) => (
                snapshot.output_identity.as_ref(),
                snapshot.continuity_token.as_ref(),
            ),
            None => stored,
        };
        let comparison = self.compare_effect(
            &effect,
            previous_output,
            previous_continuity,
            output_equivalence,
            work,
        )?;
        let pending_snapshot = if defer_snapshot_commit {
            Some(crate::logic::evaluation::PendingDependencySnapshot {
                node: effect.operational.node,
                update: effect.operational.dependency_snapshot_update.clone(),
                delta: effect.operational.snapshot_delta,
            })
        } else {
            None
        };
        Ok(ApplyCommitPacket {
            effect,
            comparison,
            pending_snapshot,
            defer_snapshot_commit,
        })
    }
}
