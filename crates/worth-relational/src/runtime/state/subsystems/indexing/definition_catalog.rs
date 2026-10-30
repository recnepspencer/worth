//! Immutable installed-definition meaning and its actual installation events.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

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
    pub(super) definitions: BTreeMap<DerivedIndexId, Arc<DerivedIndexDefinition>>,
    pub(super) field_definitions:
        BTreeMap<AspectKey, BTreeMap<FieldKey, Vec<Arc<DerivedIndexDefinition>>>>,
    introductions: BTreeMap<DerivedIndexId, u64>,
    coverage: IndexCatalogCoverage,
}
impl Default for IndexDefinitionCatalog {
    fn default() -> Self {
        Self {
            definitions: BTreeMap::new(),
            field_definitions: BTreeMap::new(),
            introductions: BTreeMap::new(),
            coverage: IndexCatalogCoverage {
                incarnation: incarnation(),
                epoch: 0,
            },
        }
    }
}
impl IndexDefinitionCatalog {
    pub(super) fn insert(&mut self, definition: DerivedIndexDefinition) {
        self.coverage.epoch = self
            .coverage
            .epoch
            .checked_add(1)
            .expect("index installation epoch exhausted");
        self.introductions
            .insert(definition.index_id, self.coverage.epoch);
        if let Some(previous) = self.definitions.remove(&definition.index_id)
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
        let definition = Arc::new(definition);
        if let Some((aspect, field)) = field_index_locator(&definition.kind) {
            self.field_definitions
                .entry(aspect)
                .or_default()
                .entry(field)
                .or_default()
                .push(Arc::clone(&definition));
        }
        self.definitions.insert(definition.index_id, definition);
    }

    pub(super) fn renew_incarnation(&mut self) {
        self.coverage.incarnation = incarnation();
    }

    pub(crate) fn coverage(&self) -> IndexCatalogCoverage {
        self.coverage
    }

    pub(super) fn with_field<R>(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
        read: impl FnOnce(&[Arc<DerivedIndexDefinition>]) -> R,
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
        use crate::history::RelationalAuthoredWriteDenial as Denial;
        // Two borrowed BTree descents, followed by full definition comparison.
        let roster = requested.locator.field_path().fields().len() as u64;
        crate::history::visit_authored_write_lookup(context, 0, roster)?;
        let width = requested
            .owned_bytes()
            .checked_add(1)
            .ok_or(Denial::Interrupted(
                worth_execution::MapKernelStop::WorkCounterOverflow,
            ))?;
        super::admitted_definition::navigation(context, self.definitions.len(), 8)?;
        let Some(actual) = self.definitions.get(&requested.index_id) else {
            return Ok(None);
        };
        crate::history::visit_authored_write_lookup(context, 0, width)?;
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
