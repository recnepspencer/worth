use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::super::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
    WorthQueryObservedRootSelection, WorthQueryObservedSource, WorthQueryObservedSourceFootprint,
};
use super::WorthQueryObservedSourceMeaning;

/// The complete selected source meaning, without its revision-bearing identity.
/// A restored or incomplete observation cannot mint this cutoff input.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedSourceSelection {
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    meaning: Arc<WorthQueryObservedSourceMeaning>,
}

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph) fn retain_selected_membership(
        &self,
    ) -> Option<WorthQueryObservedSourceSelection> {
        let footprint = self.source_meaning.footprint();
        // Selective membership cutoff has no qualified complete-walk rule yet.
        // Retained full source currentness still consumes the original bodies.
        (footprint.complete
            && footprint
                .root_selection
                .as_ref()
                .is_some_and(|selection| selection.walks.is_none()))
        .then(|| WorthQueryObservedSourceSelection {
            runtime_authority: self.runtime_authority,
            schema_binding: self.schema_binding.clone(),
            meaning: Arc::clone(&self.source_meaning),
        })
    }
}

impl WorthQueryObservedSourceSelection {
    /// Compare selected membership and contract meaning exactly. Native revisions are deliberately excluded.
    pub(in crate::domain_computation::primary_graph) fn same_selected_membership_as(
        &self,
        other: &Self,
    ) -> bool {
        if self.runtime_authority != other.runtime_authority {
            return false;
        }
        let left_binding = &self.schema_binding;
        let right_binding = &other.schema_binding;

        if left_binding.runtime_ordinal() != right_binding.runtime_ordinal() {
            return false;
        }

        if left_binding.generation() != right_binding.generation() {
            return false;
        }

        if left_binding.package_identity() != right_binding.package_identity() {
            return false;
        }

        if left_binding.schema_identity() != right_binding.schema_identity() {
            return false;
        }
        let left = &self.meaning.coordinate;
        let right = &other.meaning.coordinate;

        if left.query != right.query {
            return false;
        }

        if left.parameters != right.parameters {
            return false;
        }

        if left.occurrence != right.occurrence {
            return false;
        }

        if left.root != right.root {
            return false;
        }
        same_footprint_membership(self.meaning.footprint(), other.meaning.footprint())
    }
}

fn same_footprint_membership(
    left: &WorthQueryObservedSourceFootprint,
    right: &WorthQueryObservedSourceFootprint,
) -> bool {
    if !left.complete {
        return false;
    }

    if !right.complete {
        return false;
    }

    if left.root != right.root {
        return false;
    }

    let (Some(left_root), Some(right_root)) = (&left.root_selection, &right.root_selection) else {
        return false;
    };
    if !same_entities(&left.entities, &right.entities)
        || !same_fields(&left.aspects, &right.aspects)
        || !same_adjacencies(&left.adjacencies, &right.adjacencies)
    {
        return false;
    }
    same_root_selection(left_root, right_root)
}

fn same_root_selection(
    left: &WorthQueryObservedRootSelection,
    right: &WorthQueryObservedRootSelection,
) -> bool {
    if left.walks.is_some() || right.walks.is_some() {
        return false;
    }
    same_entities(&left.entities, &right.entities)
        && same_fields(&left.aspects, &right.aspects)
        && same_adjacencies(&left.adjacencies, &right.adjacencies)
}

fn same_entities(
    left: &[worth_relational::facade::identity::EntityId],
    right: &[worth_relational::facade::identity::EntityId],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (a, b) in left.iter().zip(right) {
        if a != b {
            return false;
        }
    }
    true
}

fn same_fields(
    left: &[WorthQueryObservedFieldRevision],
    right: &[WorthQueryObservedFieldRevision],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (a, b) in left.iter().zip(right) {
        if a.entity != b.entity {
            return false;
        }
        if !same_text(&a.entity_name, &b.entity_name)
            || !same_text(a.aspect.as_str(), b.aspect.as_str())
            || !same_text(a.field.as_str(), b.field.as_str())
        {
            return false;
        }

        if a.contract_revision != b.contract_revision {
            return false;
        }
    }
    true
}

fn same_adjacencies(
    left: &[WorthQueryObservedAdjacencyRevision],
    right: &[WorthQueryObservedAdjacencyRevision],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (a, b) in left.iter().zip(right) {
        if a.anchor != b.anchor {
            return false;
        }

        if a.relation_kind != b.relation_kind {
            return false;
        }

        if a.direction != b.direction {
            return false;
        }

        if a.comparison_work_limit != b.comparison_work_limit {
            return false;
        }
        if !same_entities(&a.endpoints, &b.endpoints) {
            return false;
        }
    }
    true
}

fn same_text(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left == right
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use worth_foundational::facade::CanonicalDigestId;
    use worth_relational::facade::identity::{EntityId, KindId, PartitionId, VersionId};

    use super::*;

    fn entity(slot: u64) -> EntityId {
        EntityId::new(PartitionId::main(), slot, 1)
    }

    fn footprint(endpoints: Vec<EntityId>, revision: u64) -> WorthQueryObservedSourceFootprint {
        let adjacency = WorthQueryObservedAdjacencyRevision {
            anchor: entity(1),
            relation_kind: KindId(7),
            direction: worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(revision)),
            comparison_work_limit: 8,
            endpoints,
        };
        WorthQueryObservedSourceFootprint {
            root: entity(1),
            complete: true,
            entities: vec![entity(1)],
            aspects: Vec::new(),
            root_selection: Some(Arc::new(WorthQueryObservedRootSelection::new(
                vec![entity(1)],
                Vec::new(),
                vec![adjacency.clone()],
                None,
                None,
            ))),
            adjacencies: vec![adjacency],
        }
    }

    impl WorthQueryObservedSourceSelection {
        /// A complete retained selection for lineage tests outside this
        /// module, which cannot observe a source of their own.
        pub(in crate::domain_computation::primary_graph) fn selecting_for_test(
            endpoint: u64,
        ) -> Self {
            let authority = crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity::mint_for_test();
            let meaning = super::super::WorthQueryObservedSourceMeaningRegistry::new(authority)
                .intern(
                    &[7; 32],
                    &[8; 32],
                    footprint(vec![entity(endpoint)], 4),
                    &super::super::WorthQueryApplicationBasisSelectionIdentity::Relational,
                )
                .unwrap();
            Self {
                runtime_authority: authority.as_u64(),
                schema_binding: ApplicationSchemaBindingIdentity::from_installed_parts(
                    1,
                    2,
                    CanonicalDigestId::new([3; 32]),
                    CanonicalDigestId::new([4; 32]),
                ),
                meaning,
            }
        }
    }

    #[test]
    fn selected_membership_ignores_revisions_but_not_selected_endpoints() {
        let before = footprint(vec![entity(2)], 4);
        let same_selection = footprint(vec![entity(2)], 9);
        let changed_selection = footprint(vec![entity(3)], 9);
        assert!(same_footprint_membership(&before, &same_selection));
        assert!(!same_footprint_membership(&before, &changed_selection,));
    }

    #[test]
    fn retained_selection_compares_a_fresh_meaning_without_revision_identity() {
        let authority = crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity::mint_for_test();
        let registry = super::super::WorthQueryObservedSourceMeaningRegistry::new(authority);
        let selection = super::super::WorthQueryApplicationBasisSelectionIdentity::Relational;
        let meaning = |endpoints, revision| {
            registry
                .intern(
                    &[7; 32],
                    &[8; 32],
                    footprint(endpoints, revision),
                    &selection,
                )
                .unwrap()
        };
        let binding = ApplicationSchemaBindingIdentity::from_installed_parts(
            1,
            2,
            CanonicalDigestId::new([3; 32]),
            CanonicalDigestId::new([4; 32]),
        );
        let carrier = |meaning| WorthQueryObservedSourceSelection {
            runtime_authority: authority.as_u64(),
            schema_binding: binding.clone(),
            meaning,
        };
        let before = carrier(meaning(vec![entity(2)], 4));
        let revision_only = carrier(meaning(vec![entity(2)], 9));
        let different = carrier(meaning(vec![entity(3)], 9));
        assert!(before.same_selected_membership_as(&revision_only));
        assert!(!before.same_selected_membership_as(&different));
    }

    #[test]
    fn selection_comparison_requires_complete_membership() {
        let selected = footprint(vec![entity(2), entity(3)], 4);
        assert!(same_footprint_membership(&selected, &selected));
        let mut incomplete = selected.clone();
        incomplete.root_selection = None;
        assert!(!same_footprint_membership(&selected, &incomplete));
    }

    #[test]
    fn text_comparison_reads_initialized_utf8_not_spare_capacity() {
        let mut spare = String::with_capacity(4096);
        spare.push_str("open");
        // Stale bytes past the initialized length stay in the spare capacity.
        let mut truncated = String::from("openXYZ");
        truncated.truncate("open".len());
        for left in [spare.as_str(), truncated.as_str()] {
            for right in ["open", "openXYZ", "opal", "closed", ""] {
                assert_eq!(same_text(left, right), left == right, "{left:?} {right:?}");
            }
        }
        assert!(same_text(&spare, &truncated));
    }
}
