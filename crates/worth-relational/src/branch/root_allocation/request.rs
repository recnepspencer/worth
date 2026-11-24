// Ordinary by-value header copies do not transfer the original allocation's
// ticket. Controlled reconstruction shares Arc owners or builds admitted ones.
impl Clone for super::super::RelationalBranchRoot {
    fn clone(&self) -> Self {
        Self {
            publication_bases: Default::default(),
            id: self.id,
            regions: self.regions.clone(),
            content: self.content.clone(),
            entity_slot_count: self.entity_slot_count,
            relation_slot_count: self.relation_slot_count,
            content_accumulator: self.content_accumulator,
            output_publication_positions: self.output_publication_positions.clone(),
            schema_authority: self.schema_authority.clone(),
            execution_contracts: self.execution_contracts.clone(),
            committed: self.committed.clone(),
            managed_source_image: self.managed_source_image.clone(),
            _memory: None,
        }
    }
}
