//! Actual immutable definition capture and source-backed identity readmission.
use super::*;
use crate::history::data::CanonicalIndexDefinitionDelta;

#[derive(Debug, Clone)]
pub(crate) struct CapturedIndexDefinitionCatalog {
    pub(crate) catalog: Arc<IndexDefinitionCatalog>,
    pub(crate) coverage: IndexCatalogCoverage,
    pub(crate) next_index_id: u64,
}
impl IndexDefinitionReadBinding {
    pub(crate) fn capture_catalog(&self) -> CapturedIndexDefinitionCatalog {
        let state = self.state.read();
        CapturedIndexDefinitionCatalog {
            catalog: state.catalog.clone(),
            coverage: state.catalog.coverage(),
            next_index_id: state.next_index_id,
        }
    }
}
impl CapturedIndexDefinitionCatalog {
    pub(crate) fn introduced(
        &self,
        base: u64,
    ) -> impl Iterator<Item = &crate::indexes::RetainedDerivedIndexDefinition> {
        self.catalog
            .definitions
            .range(DerivedIndexId(base)..)
            .map(|(_, definition)| definition)
    }
}
impl IndexingState {
    pub(crate) fn definition_frontier(&self) -> u64 {
        self.next_index_id
    }
    pub(crate) fn definition_snapshot(&self) -> Vec<DerivedIndexDefinition> {
        self.catalog
            .definitions
            .values()
            .map(|definition| definition.as_ref().clone())
            .collect()
    }
    pub(crate) fn readmit_catalog_delta(
        &mut self,
        delta: &CanonicalIndexDefinitionDelta,
    ) -> Result<(), &'static str> {
        if !delta.shape_valid() {
            return Err("invalid index definition delta");
        }
        if delta.base_next_index_id > self.next_index_id {
            return Err("index catalog delta leaves an identity gap");
        }
        for definition in delta.definitions() {
            if let Some(actual) = self.definition(definition.index_id) {
                if actual.as_ref() != definition {
                    return Err("index catalog delta conflicts with original definition");
                }
            } else {
                if definition.index_id.0 != self.next_index_id {
                    return Err("index catalog delta definition is not next");
                }
                self.insert_definition(definition.clone());
            }
        }
        self.next_index_id = self.next_index_id.max(delta.next_index_id);
        Ok(())
    }
    pub(crate) fn readmit_checkpoint_catalog(
        &mut self,
        definitions: &[DerivedIndexDefinition],
        frontier: u64,
    ) -> Result<(), &'static str> {
        if frontier == 0 || frontier.checked_sub(1) != Some(definitions.len() as u64) {
            return Err("checkpoint index catalog frontier disagrees with definitions");
        }
        for (offset, definition) in definitions.iter().enumerate() {
            if definition.index_id.0
                != (offset as u64)
                    .checked_add(1)
                    .ok_or("index catalog identity overflow")?
            {
                return Err("checkpoint index catalog is not a complete ordered prefix");
            }
            self.insert_definition(definition.clone());
        }
        self.next_index_id = frontier;
        Ok(())
    }
}
impl IndexingSubsystem {
    pub(crate) fn readmit_catalog_delta(
        &self,
        delta: &CanonicalIndexDefinitionDelta,
    ) -> Result<(), &'static str> {
        let _definitions = self.definition_publication.write();
        self.state.write().readmit_catalog_delta(delta)
    }
}
