//! Actual immutable definition capture and source-backed identity readmission.
use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::authored_index_writes::CanonicalAuthoredIndexWrites;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::index_catalog::CanonicalIndexDefinitionDelta;

#[derive(Debug, Clone)]
pub struct CapturedIndexDefinitionCatalog {
    pub catalog: Arc<IndexDefinitionCatalog>,
    pub coverage: IndexCatalogCoverage,
    pub next_index_id: u64,
}
impl IndexDefinitionReadBinding {
    pub fn capture_catalog(&self) -> CapturedIndexDefinitionCatalog {
        let state = self.state.read();
        CapturedIndexDefinitionCatalog {
            catalog: state.catalog.clone(),
            coverage: state.catalog.coverage(),
            next_index_id: state.next_index_id,
        }
    }
}
impl CapturedIndexDefinitionCatalog {
    pub fn introduced(
        &self,
        base: u64,
    ) -> impl Iterator<Item = &worth_relational_branch::internal_port::indexes::retained_definition::RetainedDerivedIndexDefinition>
{
        self.catalog
            .definitions
            .range(DerivedIndexId(base)..)
            .map(|(_, definition)| definition)
    }

    /// The only issuer of coverage for readmitted (decoded or replayed) packets.
    /// Mirrors WAL replay: the receiving catalog's coverage before the persisted
    /// bound, in this incarnation. Legacy, unavailable, mismatched and
    /// not-yet-installed bounds certify nothing; the caller marks the ordinal
    /// incomplete.
    pub fn certify_readmitted_writes(
        &self,
        writes: &CanonicalAuthoredIndexWrites,
        delta_next_index_id: u64,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        Option<IndexCatalogCoverage>,
        worth_relational_branch::internal_port::history::access::relevant_writes::RelationalAuthoredWriteDenial,
    >{
        let Some(bound) = self.readmitted_bound(writes, delta_next_index_id) else {
            return Ok(None);
        };
        // One introductions range descent.
        super::admitted_definition::navigation(context, self.catalog.definition_count(), 1)?;
        Ok(Some(self.catalog.coverage_before(bound)))
    }

    /// The bound a readmitted packet may be certified at: persisted, equal to
    /// its commit's catalog delta, and installed in this catalog.
    pub(super) fn readmitted_bound(
        &self,
        writes: &CanonicalAuthoredIndexWrites,
        delta_next_index_id: u64,
    ) -> Option<u64> {
        let Some((Some(bound), _)) = writes.persisted_entries() else {
            return None;
        };
        (bound == delta_next_index_id && bound <= self.next_index_id).then_some(bound)
    }
}
impl IndexingState {
    pub fn definition_frontier(&self) -> u64 {
        self.next_index_id
    }
    pub fn definition_snapshot(&self) -> Vec<DerivedIndexDefinition> {
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
    pub fn readmit_checkpoint_catalog(
        &mut self,
        definitions: &[DerivedIndexDefinition],
        frontier: u64,
    ) -> Result<(), &'static str> {
        // Epoch monotonicity over the restored prefix needs a fresh catalog.
        if !self.catalog.is_empty() || self.next_index_id != 1 {
            return Err("checkpoint index catalog readmits only into a fresh catalog");
        }
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
    pub fn readmit_catalog_delta(
        &self,
        delta: &CanonicalIndexDefinitionDelta,
    ) -> Result<(), &'static str> {
        let _definitions = self.definition_publication.write();
        self.state.write().readmit_catalog_delta(delta)
    }
}
