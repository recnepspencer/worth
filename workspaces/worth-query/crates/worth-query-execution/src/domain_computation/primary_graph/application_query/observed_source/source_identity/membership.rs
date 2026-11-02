use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::super::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
    WorthQueryObservedRootSelection, WorthQueryObservedSource, WorthQueryObservedSourceFootprint,
};
use super::WorthQueryObservedSourceMeaning;
use crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

/// The complete selected source meaning, without its revision-bearing identity.
/// A restored or incomplete observation cannot mint this cutoff input.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedSourceSelection {
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    meaning: Arc<WorthQueryObservedSourceMeaning>,
    managed: Option<Arc<ManagedSourceRead>>,
}

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph) fn retain_selected_membership(
        &self,
    ) -> Option<WorthQueryObservedSourceSelection> {
        let footprint = self.source_meaning.footprint();
        let selection = footprint.root_selection.as_ref()?;
        if !footprint.complete
            || selection.walks.is_some()
            || selection.derived.is_some()
            || has_derived_source_path(selection)
            || selection.components.is_some()
        {
            return None;
        }
        let managed = match (&selection.managed_index, &self.invocation_managed_source) {
            (None, None) => None,
            (Some(_), Some(actual)) if actual.has_verified_realized_footprint() => {
                Some(Arc::clone(actual))
            }
            _ => return None,
        };
        Some(WorthQueryObservedSourceSelection {
            runtime_authority: self.runtime_authority,
            schema_binding: self.schema_binding.clone(),
            meaning: Arc::clone(&self.source_meaning),
            managed,
        })
    }
}

impl WorthQueryObservedSourceSelection {
    /// Compare selected membership and contract meaning exactly. Native revisions are deliberately excluded.
    pub(in crate::domain_computation::primary_graph) fn same_selected_membership_as(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), Cause>,
    ) -> Result<bool, Cause> {
        admit(1)?;
        let equal = if self.same_coordinate(other) {
            super::admit_footprint_comparison(
                self.meaning.footprint(),
                other.meaning.footprint(),
                &mut |work| admit(u64::try_from(work).map_err(|_| Cause::WorkCounterOverflow)?),
            )?;
            if !same_footprint_membership(self.meaning.footprint(), other.meaning.footprint()) {
                false
            } else {
                match (&self.managed, &other.managed) {
                    (None, None) => true,
                    (Some(left), Some(right)) => left.same_selected_membership(right, admit)?,
                    _ => false,
                }
            }
        } else {
            false
        };
        admit(0)?;
        Ok(equal)
    }
    fn same_coordinate(&self, other: &Self) -> bool {
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
        true
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
    if left.walks.is_some()
        || left.derived.is_some()
        || right.derived.is_some()
        || has_derived_source_path(left)
        || has_derived_source_path(right)
        || right.walks.is_some()
        || left.components.is_some()
        || right.components.is_some()
    {
        return false;
    }
    same_entities(&left.entities, &right.entities)
        && same_fields(&left.aspects, &right.aspects)
        && same_adjacencies(&left.adjacencies, &right.adjacencies)
        && match (&left.managed_index, &right.managed_index) {
            (None, None) => true,
            (Some(left), Some(right)) => {
                left.access.graph_obligations().identity()
                    == right.access.graph_obligations().identity()
            }
            _ => false,
        }
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
mod tests;

// A managed Parent can hide the path inside its maintained Source contract.
// This entrance admits no selected-membership cutoff for that whole support.
fn has_derived_source_path(selection: &WorthQueryObservedRootSelection) -> bool {
    selection.managed_index.as_ref().is_some_and(|managed| {
        managed.access.meaning().source_graph().root_paths().iter()
            .flat_map(|path| path.steps()).any(|step| step.source_kind() ==
                worth_query_declaration::facade::application_query::ApplicationQueryRootPathSourceKind::Derived)
    })
}
