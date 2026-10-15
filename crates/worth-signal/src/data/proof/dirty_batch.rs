use serde::{Deserialize, Serialize};

use crate::data::aspect::{Aspect, AspectMask};
use crate::data::handle::NodeId;
use crate::data::output::CanonicalChangedRegions;

use super::{
    CanonicalForm, DedupedNodeBatch, DeltaForm, LocalityFootprint, PartitionScopeSet,
    SortedSourceBatch, SummaryForm, TouchedScopeSummary,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirtyBatchEntry {
    pub source: NodeId,
    pub changed_aspect: Aspect,
    pub changed_regions: CanonicalChangedRegions,
}

impl DirtyBatchEntry {
    pub fn new(
        source: NodeId,
        changed_aspect: Aspect,
        changed_regions: impl Into<CanonicalChangedRegions>,
    ) -> Self {
        Self {
            source,
            changed_aspect,
            changed_regions: changed_regions.into(),
        }
    }

    pub fn without_regions(source: NodeId, changed_aspect: Aspect) -> Self {
        Self::new(source, changed_aspect, CanonicalChangedRegions::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DirtyBatch {
    entries: Vec<DirtyBatchEntry>,
}

impl DirtyBatch {
    pub fn new(entries: impl IntoIterator<Item = DirtyBatchEntry>) -> Self {
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        if entries.len() > 1 {
            entries.sort_unstable_by_key(|entry| {
                (
                    entry.changed_aspect.index(),
                    super::locality::node_sort_key(&entry.source),
                )
            });
            let mut merged = Vec::<DirtyBatchEntry>::with_capacity(entries.len());
            for entry in entries {
                if let Some(previous) = merged.last_mut()
                    && previous.source == entry.source
                    && previous.changed_aspect == entry.changed_aspect
                {
                    previous.changed_regions = CanonicalChangedRegions::new(
                        previous
                            .changed_regions
                            .as_slice()
                            .iter()
                            .cloned()
                            .chain(entry.changed_regions.into_vec()),
                    );
                    continue;
                }
                merged.push(entry);
            }
            entries = merged;
        }
        Self { entries }
    }

    pub fn from_sources(entries: impl IntoIterator<Item = (NodeId, Aspect)>) -> Self {
        Self::new(entries.into_iter().map(|(source, changed_aspect)| {
            DirtyBatchEntry::without_regions(source, changed_aspect)
        }))
    }

    pub fn singleton(
        source: NodeId,
        changed_aspect: Aspect,
        changed_regions: impl Into<CanonicalChangedRegions>,
    ) -> Self {
        Self::new(std::iter::once(DirtyBatchEntry::new(
            source,
            changed_aspect,
            changed_regions,
        )))
    }

    pub fn as_slice(&self) -> &[DirtyBatchEntry] {
        &self.entries
    }

    pub fn into_vec(self) -> Vec<DirtyBatchEntry> {
        self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn changed_aspects_mask(&self) -> AspectMask {
        self.entries.iter().fold(AspectMask::EMPTY, |mask, entry| {
            mask | AspectMask::from_aspect(entry.changed_aspect)
        })
    }

    pub fn changed_regions(&self) -> CanonicalChangedRegions {
        CanonicalChangedRegions::new(
            self.entries
                .iter()
                .flat_map(|entry| entry.changed_regions.as_slice().iter().cloned()),
        )
    }

    pub fn touched_sources(&self) -> SortedSourceBatch {
        SortedSourceBatch::new(self.entries.iter().map(|entry| entry.source))
    }

    pub fn locality_footprint(&self) -> LocalityFootprint {
        let changed_regions = self.changed_regions();
        LocalityFootprint::new(
            PartitionScopeSet::from_changed_regions(&changed_regions),
            DedupedNodeBatch::new(self.entries.iter().map(|entry| entry.source)),
            self.touched_sources(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRecomputeAdmission {
    pub dirty: DirtyBatch,
    pub changed_aspects: AspectMask,
    pub changed_regions: CanonicalChangedRegions,
    pub locality: LocalityFootprint,
    pub touched_scope: TouchedScopeSummary,
    pub(crate) seeds: Vec<super::invalidation::source_seed::SourceRecomputeSeed>,
}

impl SourceRecomputeAdmission {
    pub fn new(dirty: DirtyBatch) -> Self {
        let changed_aspects = dirty.changed_aspects_mask();
        let changed_regions = dirty.changed_regions();
        let scopes = PartitionScopeSet::from_changed_regions(&changed_regions);
        let touched_nodes =
            DedupedNodeBatch::new(dirty.as_slice().iter().map(|entry| entry.source));
        let touched_sources = dirty.touched_sources();
        let locality = LocalityFootprint::new(
            scopes.clone(),
            touched_nodes.clone(),
            touched_sources.clone(),
        );
        let touched_scope =
            TouchedScopeSummary::new(scopes, touched_nodes, touched_sources.clone());
        let seeds = dirty
            .as_slice()
            .iter()
            .map(|entry| {
                super::invalidation::source_seed::SourceRecomputeSeed::new(
                    entry.source,
                    entry.changed_aspect,
                    PartitionScopeSet::from_changed_regions(&entry.changed_regions),
                )
            })
            .collect();
        Self {
            dirty,
            changed_aspects,
            changed_regions,
            locality,
            touched_scope,
            seeds,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.dirty.is_empty()
    }
}

#[deprecated(note = "use SourceRecomputeAdmission; this records root admission, not commit")]
pub type SemanticBatchCommit = SourceRecomputeAdmission;

impl CanonicalForm for CanonicalChangedRegions {}
impl DeltaForm for DirtyBatch {}
impl SummaryForm for SourceRecomputeAdmission {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_admission_retains_root_local_intent_without_descendant_authority() {
        let source = NodeId::new(1, 0);
        let aspect = Aspect::new(2);
        let admission = SourceRecomputeAdmission::new(DirtyBatch::singleton(
            source,
            aspect,
            vec![crate::data::output::ChangedRegion::new("fx")],
        ));

        assert_eq!(admission.seeds.len(), 1);
        assert_eq!(admission.seeds[0].source(), source);
        assert_eq!(admission.seeds[0].aspect(), aspect);
        assert_eq!(admission.seeds[0].changed_scopes().as_slice().len(), 1);
    }
}
