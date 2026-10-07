use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{AspectKey, FieldKey};

use super::WorthQueryPrimaryGraphLayout;

pub(in crate::domain_computation::primary_graph) enum WorthQuerySupportLookupStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

fn tree_lookup_work<Stop>(
    entries: usize,
    query_bytes: usize,
) -> Result<u64, WorthQuerySupportLookupStop<Stop>> {
    // std B-tree has at most eleven keys per node. A nonempty root has one
    // key, and every lower node has at least five keys and six children.
    let mut levels = usize::from(entries != 0);
    let mut minimum_keys = 1_usize;
    while let Some(next) = minimum_keys.checked_mul(6).and_then(|n| n.checked_add(5)) {
        if next > entries {
            break;
        }
        levels = levels
            .checked_add(1)
            .ok_or(WorthQuerySupportLookupStop::AccountingOverflow)?;
        minimum_keys = next;
    }
    let comparisons = levels
        .checked_mul(11)
        .ok_or(WorthQuerySupportLookupStop::AccountingOverflow)?;
    let comparison_bytes = query_bytes
        .checked_add(1)
        .ok_or(WorthQuerySupportLookupStop::AccountingOverflow)?;
    let work = comparisons
        .checked_mul(comparison_bytes)
        .and_then(|work| work.checked_add(1))
        .ok_or(WorthQuerySupportLookupStop::AccountingOverflow)?;
    u64::try_from(work).map_err(|_| WorthQuerySupportLookupStop::AccountingOverflow)
}

fn contains_field_admitted<Stop>(
    fields: &BTreeMap<AspectKey, BTreeSet<FieldKey>>,
    aspect: &AspectKey,
    field: &FieldKey,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<bool, WorthQuerySupportLookupStop<Stop>> {
    // Map header, query key and root-cardinality visits precede its descent.
    admit(3, 0).map_err(WorthQuerySupportLookupStop::Admission)?;
    let outer = tree_lookup_work(fields.len(), aspect.as_str().len())?;
    admit(outer, 0).map_err(WorthQuerySupportLookupStop::Admission)?;
    let Some(fields) = fields.get(aspect) else {
        return Ok(false);
    };
    // Set header, query key and cardinality visits precede its descent.
    admit(3, 0).map_err(WorthQuerySupportLookupStop::Admission)?;
    let inner = tree_lookup_work(fields.len(), field.as_str().len())?;
    admit(inner, 0).map_err(WorthQuerySupportLookupStop::Admission)?;
    Ok(fields.contains(field))
}

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn supports_relation_admitted<Stop>(
        &self,
        relation: &str,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<bool, WorthQuerySupportLookupStop<Stop>> {
        admit(3, 0).map_err(WorthQuerySupportLookupStop::Admission)?;
        let work = tree_lookup_work(self.relation_kinds.len(), relation.len())?;
        admit(work, 0).map_err(WorthQuerySupportLookupStop::Admission)?;
        Ok(self.relation(relation).is_some())
    }

    pub(in crate::domain_computation::primary_graph) fn supports_equality_field_admitted<Stop>(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<bool, WorthQuerySupportLookupStop<Stop>> {
        contains_field_admitted(&self.equality_field_keys, aspect, field, admit)
    }

    pub(in crate::domain_computation::primary_graph) fn supports_projection_field_admitted<Stop>(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<bool, WorthQuerySupportLookupStop<Stop>> {
        contains_field_admitted(&self.projection_field_keys, aspect, field, admit)
    }
}
