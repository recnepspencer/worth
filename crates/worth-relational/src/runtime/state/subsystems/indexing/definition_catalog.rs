//! Immutable installed-definition meaning and its actual installation events.
use super::*;
use crate::indexes::data::DerivedIndexKind;
use std::sync::atomic::{AtomicU64, Ordering};
mod routing;

fn incarnation() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    {
        let update_counter = &NEXT;
        let mut value = update_counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = value.checked_add(1) else {
                break Err(value);
            };
            match update_counter.compare_exchange_weak(
                value,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => value = reloaded_value,
            }
        }
    }
    .expect("index catalog incarnation exhausted")
}

/// Private provenance, never serialized or accepted from a selector. The
/// discriminator only separates actual whole-catalog installations; coverage
/// additionally requires the real definition and selected canonical interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IndexCatalogCoverage {
    incarnation: u64,
    epoch: u64,
}
impl IndexCatalogCoverage {
    pub(crate) fn extends(self, prior: Self) -> bool {
        self.incarnation == prior.incarnation && self.epoch >= prior.epoch
    }
}
impl worth_execution::ChargedBytes for IndexCatalogCoverage {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IndexDefinitionCatalog {
    pub(super) globals: im::OrdSet<DerivedIndexId>,
    pub(super) installed: im::OrdSet<DerivedIndexId>,
    pub(super) definitions:
        BTreeMap<DerivedIndexId, crate::indexes::RetainedDerivedIndexDefinition>,
    pub(super) field_definitions: BTreeMap<
        AspectKey,
        BTreeMap<FieldKey, Vec<crate::indexes::RetainedDerivedIndexDefinition>>,
    >,
    entity_field_indexes: usize,
    introductions: BTreeMap<DerivedIndexId, u64>,
    routing: routing::IndexRoutingCache,
    coverage: IndexCatalogCoverage,
    pub(super) backing: Option<Arc<super::definition_preparation::IndexDefinitionBacking>>,
}
impl Default for IndexDefinitionCatalog {
    fn default() -> Self {
        Self {
            globals: im::OrdSet::new(),
            installed: im::OrdSet::new(),
            definitions: BTreeMap::new(),
            field_definitions: BTreeMap::new(),
            entity_field_indexes: 0,
            introductions: BTreeMap::new(),
            backing: None,
            routing: routing::IndexRoutingCache::default(),
            coverage: IndexCatalogCoverage {
                incarnation: incarnation(),
                epoch: 0,
            },
        }
    }
}
impl IndexDefinitionCatalog {
    pub(super) fn insert(&mut self, definition: DerivedIndexDefinition) {
        self.installed.insert(definition.index_id);
        if definition.branch_scoped {
            self.globals.remove(&definition.index_id);
        } else {
            self.globals.insert(definition.index_id);
        }
        self.routing.clear();
        self.coverage.epoch = self
            .coverage
            .epoch
            .checked_add(1)
            .expect("index installation epoch exhausted");
        self.introductions
            .insert(definition.index_id, self.coverage.epoch);
        let previous = self.definitions.remove(&definition.index_id);
        if previous.as_ref().is_some_and(|definition| {
            matches!(definition.kind, DerivedIndexKind::EntityField { .. })
        }) {
            self.entity_field_indexes -= 1;
        }
        if matches!(definition.kind, DerivedIndexKind::EntityField { .. }) {
            self.entity_field_indexes = self
                .entity_field_indexes
                .checked_add(1)
                .expect("installed definitions fit the native catalog");
        }
        if let Some(previous) = previous
            && let Some((aspect, field)) = field_index_locator(&previous.kind)
            && let Some(fields) = self.field_definitions.get_mut(&aspect)
        {
            if let Some(definitions) = fields.get_mut(&field) {
                definitions.retain(|entry| entry.index_id != previous.index_id);
                if definitions.is_empty() {
                    fields.remove(&field);
                }
            }
            if fields.is_empty() {
                self.field_definitions.remove(&aspect);
            }
        }
        let definition = crate::indexes::RetainedDerivedIndexDefinition::funded(
            definition,
            self.backing.clone(),
        );
        if let Some((aspect, field)) = field_index_locator(&definition.kind) {
            self.field_definitions
                .entry(aspect)
                .or_default()
                .entry(field)
                .or_default()
                .push(definition.clone());
        }
        self.definitions.insert(definition.index_id, definition);
    }

    pub(super) fn renew_incarnation(&mut self) {
        self.coverage.incarnation = incarnation();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
    pub(crate) fn definition_count(&self) -> usize {
        self.definitions.len()
    }
    pub(crate) fn index_ids(&self) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.definitions.keys().copied()
    }
    pub(crate) fn definition(
        &self,
        index: DerivedIndexId,
    ) -> Option<crate::indexes::RetainedDerivedIndexDefinition> {
        self.definitions.get(&index).cloned()
    }
    pub(crate) fn has_entity_field_indexes(&self) -> bool {
        self.entity_field_indexes != 0
    }

    pub(crate) fn coverage(&self) -> IndexCatalogCoverage {
        self.coverage
    }
    /// Readmission inserts an ordered issued-ID prefix; equal overlap never
    /// reinserts it. Preserve this prefix's actual installation provenance.
    pub(crate) fn coverage_before(&self, next: u64) -> IndexCatalogCoverage {
        IndexCatalogCoverage {
            incarnation: self.coverage.incarnation,
            epoch: self
                .introductions
                .range(..DerivedIndexId(next))
                .next_back()
                .map_or(0, |(_, epoch)| *epoch),
        }
    }

    pub(crate) fn with_field<R>(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
        read: impl FnOnce(&[crate::indexes::RetainedDerivedIndexDefinition]) -> R,
    ) -> R {
        read(
            self.field_definitions
                .get(aspect)
                .and_then(|fields| fields.get(field))
                .map_or(&[], Vec::as_slice),
        )
    }

    pub(crate) fn introduction(
        &self,
        requested: &crate::history::RelationalAuthoredEntityIndexDefinition,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<IndexCatalogCoverage>, crate::history::RelationalAuthoredWriteDenial> {
        // Two borrowed BTree descents, followed by full definition comparison.
        super::admitted_definition::navigation(context, self.definitions.len(), 8)?;
        let Some(actual) = self.definitions.get(&requested.index_id) else {
            return Ok(None);
        };
        let crate::indexes::data::DerivedIndexKind::EntityField { field_locator } = &actual.kind
        else {
            return Ok(None);
        };
        if actual.name != requested.name
            || actual.branch_scoped != requested.branch_scoped
            || field_locator != &requested.locator
        {
            return Ok(None);
        }
        super::admitted_definition::navigation(context, self.introductions.len(), 8)?;
        Ok(self
            .introductions
            .get(&requested.index_id)
            .map(|epoch| IndexCatalogCoverage {
                incarnation: self.coverage.incarnation,
                epoch: *epoch,
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_meanings_with_one_version_keep_distinct_kind_routes() {
        use crate::identity::data::KindId;
        use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexKind};
        use crate::tests::support::{aspect_field_locator, aspect_key, field_key};
        let old = crate::tests::support::declared_aspect_schema_registry(
            crate::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
        );
        let mut added = old.entity_kinds.get(&KindId(1)).unwrap().clone();
        added.kind_id = KindId(91);
        added.kind_name = "new-index-covered-kind".into();
        added
            .aspect_contract_declarations
            .identity_declarations
            .clear();
        let new = old.clone().register_entity_kind(added).unwrap();
        assert_eq!(
            old.primary_schema_version_id(),
            new.primary_schema_version_id()
        );
        let indexes = IndexingSubsystem::default();
        let definition = indexes
            .register_definition(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "schema-qualified-routing".into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
            })
            .unwrap();
        indexes.install_routing(
            old.authority_digest_bytes(),
            &crate::schema::lower_aspect_plans(&old),
        );
        indexes.install_routing(
            new.authority_digest_bytes(),
            &crate::schema::lower_aspect_plans(&new),
        );
        let catalog = indexes.definition_read_binding().snapshot();
        let old_routes = catalog.routing(old.authority_digest_bytes()).unwrap();
        let new_routes = catalog.routing(new.authority_digest_bytes()).unwrap();
        assert!(old_routes.entities(KindId(91)).is_empty());
        assert_eq!(new_routes.entities(KindId(91)), &[definition.index_id]);
        assert_eq!(
            old_routes.entities(KindId(1)),
            new_routes.entities(KindId(1))
        );
    }
}
