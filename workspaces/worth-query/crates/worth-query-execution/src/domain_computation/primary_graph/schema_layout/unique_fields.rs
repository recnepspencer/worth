//! The unique fields of an installed layout, by entity kind and locator.
//!
//! A unique field's lookup is its equality index; the index id is read from
//! the field layout on every lookup, so an index not yet installed (`None`
//! until installation registers it) reads as unavailable rather than as
//! absent.

use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;

use super::{WorthQueryPrimaryFieldLayout, WorthQueryPrimaryGraphLayout};

type FieldName = (String, String, String);

#[derive(Debug, Default)]
pub(super) struct WorthQueryUniqueFieldNames {
    by_kind: BTreeMap<KindId, BTreeMap<AspectFieldLocator, FieldName>>,
}

impl WorthQueryUniqueFieldNames {
    pub(super) fn lower(fields: &BTreeMap<FieldName, WorthQueryPrimaryFieldLayout>) -> Self {
        let mut by_kind = BTreeMap::<KindId, BTreeMap<AspectFieldLocator, FieldName>>::new();
        for (name, field) in fields.iter().filter(|(_, field)| field.unique) {
            by_kind
                .entry(field.entity_kind)
                .or_default()
                .insert(field.locator.clone(), name.clone());
        }
        Self { by_kind }
    }
}

/// A unique field's installed equality index, or its absence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryUniqueFieldIndex {
    Installed(DerivedIndexId),
    Unavailable,
}

/// The unique fields one installed layout declares.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryUniqueFields<'layout> {
    names: &'layout BTreeMap<KindId, BTreeMap<AspectFieldLocator, FieldName>>,
    fields: &'layout BTreeMap<FieldName, WorthQueryPrimaryFieldLayout>,
}

impl<'layout> WorthQueryUniqueFields<'layout> {
    /// A schema that declares no unique field.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn none() -> WorthQueryUniqueFields<'static> {
        static NAMES: BTreeMap<KindId, BTreeMap<AspectFieldLocator, FieldName>> = BTreeMap::new();
        static FIELDS: BTreeMap<FieldName, WorthQueryPrimaryFieldLayout> = BTreeMap::new();
        WorthQueryUniqueFields {
            names: &NAMES,
            fields: &FIELDS,
        }
    }

    /// Whether the schema declares no unique field.
    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// The index a write of `locator` on `kind` must observe, or `None` when
    /// the field is not unique.
    pub(in crate::domain_computation::primary_graph) fn index(
        &self,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Option<WorthQueryUniqueFieldIndex> {
        let name = self.names.get(&kind)?.get(locator)?;
        Some(
            match self
                .fields
                .get(name)
                .and_then(|field| field.equality_index_id)
            {
                Some(index_id) => WorthQueryUniqueFieldIndex::Installed(index_id),
                None => WorthQueryUniqueFieldIndex::Unavailable,
            },
        )
    }
}

/// One unique field on one kind, owned, for lowering tests.
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryUniqueFieldFixture {
    names: BTreeMap<KindId, BTreeMap<AspectFieldLocator, FieldName>>,
    fields: BTreeMap<FieldName, WorthQueryPrimaryFieldLayout>,
}

#[cfg(test)]
impl WorthQueryUniqueFieldFixture {
    pub(in crate::domain_computation::primary_graph) fn new(
        kind: KindId,
        locator: AspectFieldLocator,
        equality_index_id: Option<DerivedIndexId>,
    ) -> Self {
        let name = ("Unique".to_owned(), "Identity".to_owned(), "Key".to_owned());
        let field = WorthQueryPrimaryFieldLayout {
            entity_kind: kind,
            locator,
            equality_queryable: true,
            equality_index_id,
            unique: true,
        };
        let fields = BTreeMap::from([(name, field)]);
        Self {
            names: WorthQueryUniqueFieldNames::lower(&fields).by_kind,
            fields,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn fields(
        &self,
    ) -> WorthQueryUniqueFields<'_> {
        WorthQueryUniqueFields {
            names: &self.names,
            fields: &self.fields,
        }
    }
}

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn unique_fields(
        &self,
    ) -> WorthQueryUniqueFields<'_> {
        WorthQueryUniqueFields {
            names: &self.unique_fields.by_kind,
            fields: &self.fields,
        }
    }
}
