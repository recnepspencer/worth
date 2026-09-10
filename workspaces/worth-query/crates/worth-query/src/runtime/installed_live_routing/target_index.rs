use std::collections::{BTreeMap, BTreeSet};

use crate::memory_workspace::{WorthQueryMutationDelta, WorthQueryMutationKind};

use super::WorthQueryLiveArtifactTarget;

type AspectKey = worth_foundational::facade::AspectKey;
type CollectionAspectKey = (String, AspectKey);
#[derive(Default)]
pub(super) struct WorthQueryInstalledLiveTargetIndex {
    all: BTreeMap<String, BTreeSet<WorthQueryLiveArtifactTarget>>,
    broad: BTreeMap<String, BTreeSet<WorthQueryLiveArtifactTarget>>,
    empty_touch: BTreeMap<String, BTreeSet<WorthQueryLiveArtifactTarget>>,
    structural_creation: BTreeMap<String, BTreeSet<WorthQueryLiveArtifactTarget>>,
    by_aspect: BTreeMap<CollectionAspectKey, BTreeSet<WorthQueryLiveArtifactTarget>>,
    by_whole_aspect: BTreeMap<CollectionAspectKey, BTreeSet<WorthQueryLiveArtifactTarget>>,
    by_field: BTreeMap<
        CollectionAspectKey,
        crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex<
            WorthQueryLiveArtifactTarget,
        >,
    >,
    selectors: BTreeMap<
        WorthQueryLiveArtifactTarget,
        (
            String,
            crate::domain_installation::WorthQueryInstalledLiveRoutingSelector,
        ),
    >,
}

impl WorthQueryInstalledLiveTargetIndex {
    pub(super) fn register(
        &mut self,
        target: WorthQueryLiveArtifactTarget,
        collection: String,
        selector: crate::domain_installation::WorthQueryInstalledLiveRoutingSelector,
    ) {
        self.unregister(&target);
        self.all
            .entry(collection.clone())
            .or_default()
            .insert(target.clone());
        if selector.broad {
            self.broad
                .entry(collection.clone())
                .or_default()
                .insert(target.clone());
        }
        if selector.empty_touch {
            self.empty_touch
                .entry(collection.clone())
                .or_default()
                .insert(target.clone());
        }
        if selector.structural_creation {
            self.structural_creation
                .entry(collection.clone())
                .or_default()
                .insert(target.clone());
        }
        index_many_in_collection(
            &mut self.by_aspect,
            &collection,
            &selector.aspect_routes,
            &target,
        );
        index_many_in_collection(
            &mut self.by_whole_aspect,
            &collection,
            &selector.whole_aspect_routes,
            &target,
        );
        for (aspect, path) in &selector.field_routes {
            self.by_field
                .entry((collection.clone(), aspect.clone()))
                .or_default()
                .insert(path, target.clone());
        }
        self.selectors.insert(target, (collection, selector));
    }

    pub(super) fn unregister(&mut self, target: &WorthQueryLiveArtifactTarget) {
        let Some((collection, selector)) = self.selectors.remove(target) else {
            return;
        };
        remove_target(&mut self.all, &collection, target);
        remove_target(&mut self.broad, &collection, target);
        remove_target(&mut self.empty_touch, &collection, target);
        remove_target(&mut self.structural_creation, &collection, target);
        unindex_many_in_collection(
            &mut self.by_aspect,
            &collection,
            &selector.aspect_routes,
            target,
        );
        unindex_many_in_collection(
            &mut self.by_whole_aspect,
            &collection,
            &selector.whole_aspect_routes,
            target,
        );
        for (aspect, path) in &selector.field_routes {
            let key = (collection.clone(), aspect.clone());
            if let Some(index) = self.by_field.get_mut(&key) {
                index.remove(path, target);
            }
        }
        self.by_field.retain(|_, index| !index.is_empty());
    }

    pub(super) fn affected_targets(
        &self,
        mutation: &WorthQueryMutationDelta,
    ) -> WorthQueryInstalledTargetSelection {
        let mut work = WorthQueryInstalledTargetSelectionWork {
            collection_index_probes: 1,
            ..Default::default()
        };
        let collection = mutation.target_collection_identity().as_str();
        if mutation.kind() == &WorthQueryMutationKind::Deleted {
            work.relevance_index_probes += 1;
            return selection(self.all.get(collection).cloned().unwrap_or_default(), work);
        }
        work.relevance_index_probes += 1;
        let mut targets = self.broad.get(collection).cloned().unwrap_or_default();
        if mutation.kind() == &WorthQueryMutationKind::Created {
            work.relevance_index_probes += 1;
            extend(&mut targets, self.structural_creation.get(collection));
        }
        if mutation.admitted_touched_aspects().is_empty() {
            work.relevance_index_probes += 1;
            extend(&mut targets, self.empty_touch.get(collection));
        }
        for touch in mutation.admitted_touched_aspects() {
            let aspect_key = (collection.to_owned(), touch.native_aspect_key().clone());
            if let Some(path) = touch.native_field_path() {
                work.relevance_index_probes += 1;
                extend(&mut targets, self.by_whole_aspect.get(&aspect_key));
                work.relevance_index_probes += 1;
                if let Some(index) = self.by_field.get(&aspect_key) {
                    let (overlapping, path_work) = index.overlapping(path);
                    work.relevance_index_probes += path_work.node_probes;
                    work.overlap_deduplications += path_work.overlap_deduplications;
                    insert_targets(&mut targets, &overlapping, &mut work);
                }
            } else {
                work.relevance_index_probes += 1;
                extend(&mut targets, self.by_aspect.get(&aspect_key));
            }
        }
        selection(targets, work)
    }
}

pub(crate) struct WorthQueryInstalledTargetSelection {
    pub(crate) targets: BTreeSet<WorthQueryLiveArtifactTarget>,
    pub(crate) work: WorthQueryInstalledTargetSelectionWork,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct WorthQueryInstalledTargetSelectionWork {
    pub(crate) collection_index_probes: usize,
    pub(crate) relevance_index_probes: usize,
    pub(crate) candidates_selected: usize,
    pub(crate) overlap_deduplications: usize,
}

fn selection(
    targets: BTreeSet<WorthQueryLiveArtifactTarget>,
    mut work: WorthQueryInstalledTargetSelectionWork,
) -> WorthQueryInstalledTargetSelection {
    work.candidates_selected = targets.len();
    WorthQueryInstalledTargetSelection { targets, work }
}

fn insert_targets(
    target: &mut BTreeSet<WorthQueryLiveArtifactTarget>,
    source: &BTreeSet<WorthQueryLiveArtifactTarget>,
    work: &mut WorthQueryInstalledTargetSelectionWork,
) {
    for value in source {
        if !target.insert(value.clone()) {
            work.overlap_deduplications += 1;
        }
    }
}

fn index_many_in_collection<K: Ord + Clone>(
    index: &mut BTreeMap<(String, K), BTreeSet<WorthQueryLiveArtifactTarget>>,
    collection: &str,
    keys: &BTreeSet<K>,
    target: &WorthQueryLiveArtifactTarget,
) {
    for key in keys {
        index
            .entry((collection.to_owned(), key.clone()))
            .or_default()
            .insert(target.clone());
    }
}

fn unindex_many_in_collection<K: Ord + Clone>(
    index: &mut BTreeMap<(String, K), BTreeSet<WorthQueryLiveArtifactTarget>>,
    collection: &str,
    keys: &BTreeSet<K>,
    target: &WorthQueryLiveArtifactTarget,
) {
    for key in keys {
        remove_target(index, &(collection.to_owned(), key.clone()), target);
    }
}

fn remove_target<K: Ord + Clone>(
    index: &mut BTreeMap<K, BTreeSet<WorthQueryLiveArtifactTarget>>,
    key: &K,
    target: &WorthQueryLiveArtifactTarget,
) {
    let remove_key = index.get_mut(key).is_some_and(|targets| {
        targets.remove(target);
        targets.is_empty()
    });
    if remove_key {
        index.remove(key);
    }
}

fn extend(
    target: &mut BTreeSet<WorthQueryLiveArtifactTarget>,
    source: Option<&BTreeSet<WorthQueryLiveArtifactTarget>>,
) {
    if let Some(source) = source {
        target.extend(source.iter().cloned());
    }
}

#[cfg(test)]
#[path = "target_index_tests.rs"]
mod tests;
