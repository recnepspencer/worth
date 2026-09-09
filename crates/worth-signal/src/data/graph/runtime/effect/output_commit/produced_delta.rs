//! Output delta derivation with admitted region copies and canonicalization.
use super::super::region_preparation::copy_region_scopes;
use super::{
    ApplyCommitPacket, Aspect, AspectMask, EvaluationVerdict, EvaluationWork, ProducedAspectDelta,
    SignalError, SignalGraph,
};
use crate::data::proof::invalidation::output_commit::{
    NonEmptyCanonicalAspectChangeSet, ProducedAspectChange, ScopePrecision,
};

impl SignalGraph {
    pub(super) fn prepare_produced_delta(
        &self,
        apply: &ApplyCommitPacket,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Option<ProducedAspectDelta>, SignalError> {
        if !matches!(
            apply.effect.operational.verdict,
            EvaluationVerdict::Recomputed
        ) || apply.comparison.propagation_suppressed
        {
            return Ok(None);
        }
        let producer = apply.effect.operational.node;
        let previous = self.node_aspect_version(producer)?;
        let committed = apply.effect.operational.aspect_version;
        let produces = self.get_contract(producer)?.semantics.produces;
        let exact = apply.effect.changed_aspect_regions();
        let legacy = apply.effect.changed_regions();
        let aspects = crate::data::aspect::MAX_ASPECTS;
        // Two exact-region passes per emitted aspect: count, then select.
        // Fixed-domain allocation, version scans and aspect canonicalization
        // are included before collecting any changes.
        work.reserve(
            exact
                .len()
                .checked_mul(2)
                .and_then(|n| n.checked_mul(aspects))
                .and_then(|n| {
                    n.checked_add(
                        aspects * std::mem::size_of::<ProducedAspectChange>()
                            + 8 * aspects * aspects
                            + 8 * aspects,
                    )
                }),
        )?;
        let changed_count = previous
            .slots()
            .iter()
            .zip(committed.slots())
            .filter(|(before, after)| before != after)
            .count();
        let scope_precision = if !legacy.is_empty() && changed_count > 1 {
            ScopePrecision::ConservativeLegacyUnion
        } else {
            ScopePrecision::ExactAspectScopes
        };
        let mut changes = Vec::with_capacity(aspects);
        for (index, (&before, &after)) in previous.slots().iter().zip(committed.slots()).enumerate()
        {
            let aspect = Aspect::new(index as u8);
            if before == after || !produces.contains(AspectMask::from_aspect(aspect)) {
                continue;
            }
            let exact_count = exact.iter().filter(|(owner, _)| *owner == aspect).count();
            let legacy_count = if exact_count == 0 { legacy.len() } else { 0 };
            let regions = exact
                .iter()
                .filter(|(owner, _)| *owner == aspect)
                .map(|(_, region)| region)
                .chain(legacy.iter().take(legacy_count));
            let changed_scopes = copy_region_scopes(regions, exact_count + legacy_count, work)?;
            changes.push(ProducedAspectChange {
                aspect,
                previous_version: before,
                committed_version: after,
                changed_scopes,
            });
        }
        Ok(
            NonEmptyCanonicalAspectChangeSet::new(changes).map(|changes| ProducedAspectDelta {
                producer,
                output_commit_ordinal: self.cause_sets.reserve_output_commit_ordinal(),
                committed_output_version: committed,
                changes,
                scope_precision,
            }),
        )
    }
}

#[cfg(test)]
mod tests;
