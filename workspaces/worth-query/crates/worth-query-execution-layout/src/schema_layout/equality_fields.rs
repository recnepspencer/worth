use std::borrow::Borrow;
use std::cmp::Ordering;

use worth_relational::facade::indexes::DerivedIndexId;

use super::{WorthQueryPrimaryFieldLayout, WorthQueryPrimaryGraphLayout};

trait FieldTripleKey {
    fn triple(&self) -> (&str, &str, &str);
}

impl FieldTripleKey for (String, String, String) {
    fn triple(&self) -> (&str, &str, &str) {
        (self.0.as_str(), self.1.as_str(), self.2.as_str())
    }
}

impl FieldTripleKey for (&str, &str, &str) {
    fn triple(&self) -> (&str, &str, &str) {
        *self
    }
}

impl PartialEq for dyn FieldTripleKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.triple() == other.triple()
    }
}

impl Eq for dyn FieldTripleKey + '_ {}

impl PartialOrd for dyn FieldTripleKey + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn FieldTripleKey + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.triple().cmp(&other.triple())
    }
}

impl<'a> Borrow<dyn FieldTripleKey + 'a> for (String, String, String) {
    fn borrow(&self) -> &(dyn FieldTripleKey + 'a) {
        self
    }
}

impl WorthQueryPrimaryGraphLayout {
    pub fn equality_field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryPrimaryFieldLayout> {
        let key: &dyn FieldTripleKey = &(entity, aspect, field);
        self.fields
            .get(key)
            .filter(|layout| layout.equality_queryable)
    }

    pub fn equality_selector_index(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<DerivedIndexId> {
        self.equality_field(entity, aspect, field)
            .and_then(|layout| layout.equality_index_id)
    }

    pub fn equality_fields_mut(
        &mut self,
    ) -> impl Iterator<Item = (&(String, String, String), &mut WorthQueryPrimaryFieldLayout)> {
        self.fields
            .iter_mut()
            .filter(|(_, layout)| layout.equality_queryable)
    }

    pub fn equality_index_ids(&self) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.fields
            .values()
            .filter_map(|field| field.equality_index_id)
    }
}

mod court_doors_for_test;
