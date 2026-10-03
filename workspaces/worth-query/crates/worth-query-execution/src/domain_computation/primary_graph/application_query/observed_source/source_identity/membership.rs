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
        (footprint.complete && footprint.root_selection.is_some()).then(|| {
            WorthQueryObservedSourceSelection {
                runtime_authority: self.runtime_authority,
                schema_binding: self.schema_binding.clone(),
                meaning: Arc::clone(&self.source_meaning),
            }
        })
    }
}

impl WorthQueryObservedSourceSelection {
    /// Compare selected membership and contract meaning under the caller's
    /// cumulative Work admission. Native revisions are deliberately excluded.
    pub(in crate::domain_computation::primary_graph) fn same_selected_membership_as<E>(
        &self,
        other: &Self,
        mut admit: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        admit(1)?;
        if self.runtime_authority != other.runtime_authority {
            return Ok(false);
        }
        let left_binding = &self.schema_binding;
        let right_binding = &other.schema_binding;
        admit(1)?;
        if left_binding.runtime_ordinal() != right_binding.runtime_ordinal() {
            return Ok(false);
        }
        admit(1)?;
        if left_binding.generation() != right_binding.generation() {
            return Ok(false);
        }
        admit(1)?;
        if left_binding.package_identity() != right_binding.package_identity() {
            return Ok(false);
        }
        admit(1)?;
        if left_binding.schema_identity() != right_binding.schema_identity() {
            return Ok(false);
        }
        let left = &self.meaning.coordinate;
        let right = &other.meaning.coordinate;
        admit(1)?;
        if left.query != right.query {
            return Ok(false);
        }
        admit(1)?;
        if left.parameters != right.parameters {
            return Ok(false);
        }
        admit(1)?;
        if left.occurrence != right.occurrence {
            return Ok(false);
        }
        admit(1)?;
        if left.root != right.root {
            return Ok(false);
        }
        same_footprint_membership(
            self.meaning.footprint(),
            other.meaning.footprint(),
            &mut admit,
        )
    }
}

fn same_footprint_membership<E>(
    left: &WorthQueryObservedSourceFootprint,
    right: &WorthQueryObservedSourceFootprint,
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    admit(1)?;
    if !left.complete {
        return Ok(false);
    }
    admit(1)?;
    if !right.complete {
        return Ok(false);
    }
    admit(1)?;
    if left.root != right.root {
        return Ok(false);
    }
    admit(1)?;
    let (Some(left_root), Some(right_root)) = (&left.root_selection, &right.root_selection) else {
        return Ok(false);
    };
    if !same_entities(&left.entities, &right.entities, admit)?
        || !same_fields(&left.aspects, &right.aspects, admit)?
        || !same_adjacencies(&left.adjacencies, &right.adjacencies, admit)?
    {
        return Ok(false);
    }
    same_root_selection(left_root, right_root, admit)
}

fn same_root_selection<E>(
    left: &WorthQueryObservedRootSelection,
    right: &WorthQueryObservedRootSelection,
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    Ok(same_entities(&left.entities, &right.entities, admit)?
        && same_fields(&left.aspects, &right.aspects, admit)?
        && same_adjacencies(&left.adjacencies, &right.adjacencies, admit)?)
}

fn same_entities<E>(
    left: &[worth_relational::facade::identity::EntityId],
    right: &[worth_relational::facade::identity::EntityId],
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    admit(1)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (a, b) in left.iter().zip(right) {
        admit(1)?;
        if a != b {
            return Ok(false);
        }
    }
    Ok(true)
}

fn same_fields<E>(
    left: &[WorthQueryObservedFieldRevision],
    right: &[WorthQueryObservedFieldRevision],
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    admit(1)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (a, b) in left.iter().zip(right) {
        admit(1)?;
        if a.entity != b.entity {
            return Ok(false);
        }
        if !same_text(&a.entity_name, &b.entity_name, admit)?
            || !same_text(a.aspect.as_str(), b.aspect.as_str(), admit)?
            || !same_text(a.field.as_str(), b.field.as_str(), admit)?
        {
            return Ok(false);
        }
        admit(1)?;
        if a.contract_revision != b.contract_revision {
            return Ok(false);
        }
    }
    Ok(true)
}

fn same_adjacencies<E>(
    left: &[WorthQueryObservedAdjacencyRevision],
    right: &[WorthQueryObservedAdjacencyRevision],
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    admit(1)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (a, b) in left.iter().zip(right) {
        admit(1)?;
        if a.anchor != b.anchor {
            return Ok(false);
        }
        admit(1)?;
        if a.relation_kind != b.relation_kind {
            return Ok(false);
        }
        admit(1)?;
        if a.direction != b.direction {
            return Ok(false);
        }
        admit(1)?;
        if a.comparison_work_limit != b.comparison_work_limit {
            return Ok(false);
        }
        if !same_entities(&a.endpoints, &b.endpoints, admit)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn same_text<E>(
    left: &str,
    right: &str,
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    admit(1)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    admit(left.len())?;
    Ok(left == right)
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
            ))),
            adjacencies: vec![adjacency],
        }
    }

    #[test]
    fn selected_membership_ignores_revisions_but_not_selected_endpoints() {
        let before = footprint(vec![entity(2)], 4);
        let same_selection = footprint(vec![entity(2)], 9);
        let changed_selection = footprint(vec![entity(3)], 9);
        assert!(
            same_footprint_membership(&before, &same_selection, &mut |_| Ok::<_, ()>(())).unwrap()
        );
        assert!(
            !same_footprint_membership(&before, &changed_selection, &mut |_| Ok::<_, ()>(()),)
                .unwrap()
        );
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
        assert!(before
            .same_selected_membership_as(&revision_only, |_| Ok::<_, ()>(()))
            .unwrap());
        assert!(!before
            .same_selected_membership_as(&different, |_| Ok::<_, ()>(()))
            .unwrap());
    }

    #[test]
    fn selection_comparison_stops_at_the_caller_work_boundary() {
        let selected = footprint(vec![entity(2), entity(3)], 4);
        let mut remaining = 2usize;
        assert!(
            same_footprint_membership(&selected, &selected, &mut |units| {
                remaining = remaining.checked_sub(units).ok_or("work exhausted")?;
                Ok::<_, &'static str>(())
            })
            .is_err()
        );
        let mut incomplete = selected.clone();
        incomplete.root_selection = None;
        assert!(
            !same_footprint_membership(&selected, &incomplete, &mut |_| Ok::<_, ()>(())).unwrap()
        );
    }

    #[test]
    fn text_comparison_meters_initialized_utf8_not_spare_capacity() {
        let mut spare = String::with_capacity(4096);
        spare.push_str("open");
        let mut work = 0;
        assert!(same_text(&spare, "open", &mut |units| {
            work += units;
            Ok::<_, ()>(())
        })
        .unwrap());
        assert_eq!(work, 1 + "open".len());
        let mut work = 0;
        assert!(!same_text(&spare, "closed", &mut |units| {
            work += units;
            Ok::<_, ()>(())
        })
        .unwrap());
        assert_eq!(work, 1, "different lengths compare without reading payload");
    }
}
