//! Test-only reference model: independent map grouping and library sorting.
//! Production output preparation lives in the runtime output-commit owner.
use super::*;
use crate::data::output::{CanonicalChangedRegions, ChangedRegion};
use std::collections::BTreeMap;

impl NonEmptyCanonicalAspectChangeSet {
    pub(crate) fn from_versions_and_regions(
        previous: AspectVersion,
        candidate: AspectVersion,
        exact_regions: &[(Aspect, ChangedRegion)],
        legacy_regions: &[ChangedRegion],
    ) -> Option<(Self, ScopePrecision)> {
        let exact = exact_regions.iter().fold(
            BTreeMap::<Aspect, Vec<ChangedRegion>>::new(),
            |mut grouped, (aspect, region)| {
                grouped.entry(*aspect).or_default().push(region.clone());
                grouped
            },
        );
        let changed = previous
            .slots()
            .iter()
            .zip(candidate.slots())
            .enumerate()
            .filter_map(|(index, (&before, &after))| {
                (before != after).then_some((Aspect::new(index as u8), before, after))
            })
            .collect::<Vec<_>>();
        let precision = if !legacy_regions.is_empty() && changed.len() > 1 {
            ScopePrecision::ConservativeLegacyUnion
        } else {
            ScopePrecision::ExactAspectScopes
        };
        let legacy = CanonicalChangedRegions::new(legacy_regions.to_vec());
        let changes = changed
            .into_iter()
            .map(|(aspect, previous_version, committed_version)| {
                let regions = exact
                    .get(&aspect)
                    .cloned()
                    .map(CanonicalChangedRegions::new)
                    .unwrap_or_else(|| legacy.clone());
                ProducedAspectChange {
                    aspect,
                    previous_version,
                    committed_version,
                    changed_scopes: PartitionScopeSet::from_changed_regions(&regions),
                }
            })
            .collect();
        Self::new(changes).map(|changes| (changes, precision))
    }
}

impl ProducedAspectDelta {
    pub(crate) fn from_committed_result(
        producer: NodeId,
        output_commit_ordinal: OutputCommitOrdinal,
        previous: AspectVersion,
        committed: AspectVersion,
        produced_aspects: AspectMask,
        exact_regions: &[(Aspect, ChangedRegion)],
        legacy_regions: &[ChangedRegion],
    ) -> Option<Self> {
        let (changes, scope_precision) =
            NonEmptyCanonicalAspectChangeSet::from_versions_and_regions(
                previous,
                committed,
                exact_regions,
                legacy_regions,
            )?;
        let changes = NonEmptyCanonicalAspectChangeSet::new(
            changes
                .as_slice()
                .iter()
                .filter(|change| produced_aspects.contains(AspectMask::from_aspect(change.aspect)))
                .cloned()
                .collect(),
        )?;
        Some(Self {
            producer,
            output_commit_ordinal,
            committed_output_version: committed,
            changes,
            scope_precision,
        })
    }
}
