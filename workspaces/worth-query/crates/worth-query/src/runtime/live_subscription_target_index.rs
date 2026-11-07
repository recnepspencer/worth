use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::{BTreeMap, BTreeSet};

use crate::declarative_live::{DeclarativeLiveQueryRequest, DeclarativeLiveViewShape};
use crate::memory_workspace::WorthQueryMutationDelta;

use super::WorthQueryLiveArtifactTarget;

#[derive(Default)]
pub(super) struct WorthQueryLiveSubscriptionTargetIndex {
    collections: BTreeMap<String, WorthQueryCollectionLiveTargets>,
    routes_by_target: BTreeMap<WorthQueryLiveArtifactTarget, (String, WorthQueryLiveRequestRoutes)>,
}

#[derive(Default)]
struct WorthQueryCollectionLiveTargets {
    all: BTreeSet<WorthQueryLiveArtifactTarget>,
    by_aspect:
        BTreeMap<worth_foundational::facade::AspectKey, BTreeSet<WorthQueryLiveArtifactTarget>>,
    by_whole_aspect:
        BTreeMap<worth_foundational::facade::AspectKey, BTreeSet<WorthQueryLiveArtifactTarget>>,
    by_field: BTreeMap<
        worth_foundational::facade::AspectKey,
        crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex<
            WorthQueryLiveArtifactTarget,
        >,
    >,
}

impl WorthQueryLiveSubscriptionTargetIndex {
    pub(super) fn register(
        &mut self,
        target: WorthQueryLiveArtifactTarget,
        request: &DeclarativeLiveQueryRequest,
    ) -> Result<(), IdentityDerivationDenial> {
        self.unregister(&target);
        let collection = request.target_collection_identity()?.as_str().to_owned();
        let entry = self.collections.entry(collection.clone()).or_default();
        entry.all.insert(target.clone());
        let routes = request_routes(request);
        for aspect in &routes.aspects {
            entry
                .by_aspect
                .entry(aspect.clone())
                .or_default()
                .insert(target.clone());
        }
        for aspect in &routes.whole_aspects {
            entry
                .by_whole_aspect
                .entry(aspect.clone())
                .or_default()
                .insert(target.clone());
        }
        for (aspect, path) in &routes.fields {
            entry
                .by_field
                .entry(aspect.clone())
                .or_default()
                .insert(path, target.clone());
        }
        self.routes_by_target.insert(target, (collection, routes));

        Ok(())
    }

    pub(super) fn unregister(&mut self, target: &WorthQueryLiveArtifactTarget) {
        let Some((collection, routes)) = self.routes_by_target.remove(target) else {
            return;
        };
        let remove_collection = self.collections.get_mut(&collection).is_some_and(|entry| {
            entry.all.remove(target);
            unindex_sets(&mut entry.by_aspect, &routes.aspects, target);
            unindex_sets(&mut entry.by_whole_aspect, &routes.whole_aspects, target);
            for (aspect, path) in routes.fields {
                if let Some(index) = entry.by_field.get_mut(&aspect) {
                    index.remove(&path, target);
                }
            }
            entry.by_field.retain(|_, index| !index.is_empty());
            entry.all.is_empty()
        });
        if remove_collection {
            self.collections.remove(&collection);
        }
    }

    pub(super) fn affected_targets(
        &self,
        mutation: &WorthQueryMutationDelta,
    ) -> WorthQueryLiveTargetSelection {
        let mut work = WorthQueryLiveTargetSelectionWork {
            collection_index_probes: 1,
            ..Default::default()
        };
        let Some(entry) = self
            .collections
            .get(mutation.target_collection_identity().as_str())
        else {
            return selection(BTreeSet::new(), work);
        };
        if mutation.admitted_touched_aspects().is_empty()
            || matches!(
                mutation.kind(),
                crate::memory_workspace::WorthQueryMutationKind::Created
                    | crate::memory_workspace::WorthQueryMutationKind::Deleted
            )
        {
            work.relevance_index_probes += 1;
            return selection(entry.all.clone(), work);
        }
        let mut targets = BTreeSet::new();
        for touch in mutation.admitted_touched_aspects() {
            let aspect = touch.native_aspect_key();
            match touch.native_field_path() {
                Some(path) => {
                    work.relevance_index_probes += 1;
                    extend(&mut targets, entry.by_whole_aspect.get(aspect));
                    work.relevance_index_probes += 1;
                    if let Some(index) = entry.by_field.get(aspect) {
                        let (overlapping, path_work) = index.overlapping(path);
                        work.relevance_index_probes += path_work.node_probes;
                        work.overlap_deduplications += path_work.overlap_deduplications;
                        insert_targets(&mut targets, &overlapping, &mut work);
                    }
                }
                None => {
                    work.relevance_index_probes += 1;
                    extend(&mut targets, entry.by_aspect.get(aspect));
                }
            }
        }
        selection(targets, work)
    }
}

#[derive(Clone, Default)]
struct WorthQueryLiveRequestRoutes {
    aspects: BTreeSet<worth_foundational::facade::AspectKey>,
    whole_aspects: BTreeSet<worth_foundational::facade::AspectKey>,
    fields: BTreeSet<(
        worth_foundational::facade::AspectKey,
        worth_foundational::facade::CanonicalFieldPath,
    )>,
}

fn request_routes(request: &DeclarativeLiveQueryRequest) -> WorthQueryLiveRequestRoutes {
    let mut routes = WorthQueryLiveRequestRoutes::default();
    for field in request
        .query_projection()
        .iter()
        .map(|projection| projection.source_field_key())
        .chain(
            request
                .predicate_filters()
                .iter()
                .map(|filter| filter.source_field_key()),
        )
        .chain(
            request
                .ordering()
                .iter()
                .map(|ordering| ordering.source_field_key()),
        )
    {
        let aspect = field.native_aspect_key();
        routes.aspects.insert(aspect.clone());
        routes.fields.insert((
            aspect,
            worth_foundational::facade::CanonicalFieldPath::single(field.native_field_key()),
        ));
    }
    match request.view_shape() {
        DeclarativeLiveViewShape::InspectorFocused { focused_aspect }
        | DeclarativeLiveViewShape::IdentityAwareInspectorFocused { focused_aspect, .. } => {
            routes.aspects.insert(focused_aspect.clone());
            routes.whole_aspects.insert(focused_aspect.clone());
        }
        DeclarativeLiveViewShape::KanbanGrouped { grouping_aspect } => {
            routes.aspects.insert(grouping_aspect.clone());
            routes.whole_aspects.insert(grouping_aspect.clone());
        }
        _ => {}
    }
    routes
}

pub(crate) struct WorthQueryLiveTargetSelection {
    pub(crate) targets: BTreeSet<WorthQueryLiveArtifactTarget>,
    pub(crate) work: WorthQueryLiveTargetSelectionWork,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct WorthQueryLiveTargetSelectionWork {
    pub(crate) collection_index_probes: usize,
    pub(crate) relevance_index_probes: usize,
    pub(crate) candidates_selected: usize,
    pub(crate) overlap_deduplications: usize,
}

fn selection(
    targets: BTreeSet<WorthQueryLiveArtifactTarget>,
    mut work: WorthQueryLiveTargetSelectionWork,
) -> WorthQueryLiveTargetSelection {
    work.candidates_selected = targets.len();
    WorthQueryLiveTargetSelection { targets, work }
}

fn insert_targets(
    target: &mut BTreeSet<WorthQueryLiveArtifactTarget>,
    source: &BTreeSet<WorthQueryLiveArtifactTarget>,
    work: &mut WorthQueryLiveTargetSelectionWork,
) {
    for value in source {
        if !target.insert(value.clone()) {
            work.overlap_deduplications += 1;
        }
    }
}

fn unindex_sets<K: Ord + Clone>(
    index: &mut BTreeMap<K, BTreeSet<WorthQueryLiveArtifactTarget>>,
    keys: &BTreeSet<K>,
    target: &WorthQueryLiveArtifactTarget,
) {
    for key in keys {
        let remove = index.get_mut(key).is_some_and(|targets| {
            targets.remove(target);
            targets.is_empty()
        });
        if remove {
            index.remove(key);
        }
    }
}

fn extend(
    targets: &mut BTreeSet<WorthQueryLiveArtifactTarget>,
    indexed: Option<&BTreeSet<WorthQueryLiveArtifactTarget>>,
) {
    if let Some(indexed) = indexed {
        targets.extend(indexed.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative_live::{
        DeclarativeEqualityFilter, DeclarativeLiveViewShape, DeclarativeOrderingField,
        DeclarativeProjectionField,
    };

    #[test]
    fn collection_field_and_whole_aspect_indexes_ignore_unrelated_target_scale(
    ) -> Result<(), IdentityDerivationDenial> {
        let mut index = WorthQueryLiveSubscriptionTargetIndex::default();
        for ordinal in 0..64 {
            index.register(
                WorthQueryLiveArtifactTarget::from_view_name(format!("other-{ordinal}")),
                &request("Other", "noise", "value"),
            );
            index.register(
                WorthQueryLiveArtifactTarget::from_view_name(format!("vertex-noise-{ordinal}")),
                &request("Vertex", "noise", "value"),
            );
            index.register(
                WorthQueryLiveArtifactTarget::from_view_name(format!("vertex-name-{ordinal}")),
                &request("Vertex", "identity", "name"),
            );
        }
        let first = WorthQueryLiveArtifactTarget::from_view_name("affected-first");
        let second = WorthQueryLiveArtifactTarget::from_view_name("affected-second");
        index.register(first.clone(), &request("Vertex", "identity", "id"));
        index.register(second.clone(), &request("Vertex", "identity", "id"));

        let affected = index
            .affected_targets(&field_mutation("Vertex", "identity", &["id"])?)
            .targets;
        assert_eq!(affected, BTreeSet::from([first, second]));

        let whole_aspect = index
            .affected_targets(&whole_aspect_mutation("Vertex", "identity")?)
            .targets;
        assert_eq!(whole_aspect.len(), 66);

        Ok(())
    }

    #[test]
    fn predicate_and_ordering_dependencies_route_when_projection_omits_them(
    ) -> Result<(), IdentityDerivationDenial> {
        let target = WorthQueryLiveArtifactTarget::from_view_name("filtered-and-ordered");
        let request = request("Vertex", "identity", "id")
            .where_equal(DeclarativeEqualityFilter::new(
                authoring_field("status", "state"),
                crate::authoring::WorthQueryPredicateOperand::string("open".to_owned()),
            ))
            .order_by_direction(DeclarativeOrderingField::ascending(authoring_field(
                "ranking", "position",
            )));
        let mut index = WorthQueryLiveSubscriptionTargetIndex::default();
        index.register(target.clone(), &request);

        assert_eq!(
            index
                .affected_targets(&field_mutation("Vertex", "status", &["state"])?)
                .targets,
            BTreeSet::from([target.clone()])
        );
        assert_eq!(
            index
                .affected_targets(&field_mutation("Vertex", "ranking", &["position"])?)
                .targets,
            BTreeSet::from([target])
        );

        Ok(())
    }

    fn request(collection: &str, aspect: &str, field: &str) -> DeclarativeLiveQueryRequest {
        DeclarativeLiveQueryRequest::new(collection, DeclarativeLiveViewShape::table()).project(
            DeclarativeProjectionField::from_authoring_parts(aspect, field)
                .delivered_as(format!("{aspect}.{field}")),
        )
    }

    fn authoring_field(aspect: &str, field: &str) -> crate::authoring::AspectFieldKey {
        DeclarativeProjectionField::from_authoring_parts(aspect, field)
            .source_field_key()
            .clone()
    }

    fn whole_aspect_mutation(
        collection: &str,
        aspect: &str,
    ) -> Result<WorthQueryMutationDelta, IdentityDerivationDenial> {
        Ok(WorthQueryMutationDelta::from_touched_aspects(
            collection,
            crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 1, 1),
            ),
            crate::memory_workspace::WorthQueryMutationKind::Updated,
            vec![crate::runtime::WorthQueryAspectTouch::whole_aspect(
                worth_foundational::facade::AspectKey::new(aspect).unwrap(),
            )],
        )?)
    }

    fn field_mutation(
        collection: &str,
        aspect: &str,
        fields: &[&str],
    ) -> Result<WorthQueryMutationDelta, IdentityDerivationDenial> {
        Ok(WorthQueryMutationDelta::from_touched_aspects(
            collection,
            crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 1, 1),
            ),
            crate::memory_workspace::WorthQueryMutationKind::Updated,
            vec![crate::runtime::WorthQueryAspectTouch::aspect_field_path(
                worth_foundational::facade::AspectKey::new(aspect).unwrap(),
                worth_foundational::facade::CanonicalFieldPath::new(
                    fields
                        .iter()
                        .map(|field| worth_foundational::facade::FieldKey::new(*field).unwrap()),
                )
                .unwrap(),
            )],
        )?)
    }
}
