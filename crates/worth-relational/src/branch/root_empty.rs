//! Empty roots carry schema DATA without claiming an admitted Home executor.
use super::*;
use std::collections::BTreeMap;

impl RelationalBranchRoot {
    #[cfg(test)]
    pub(crate) fn empty() -> Arc<Self> {
        let issuer = RelationalBranchRootIdentityIssuer::default();
        Arc::new(Self {
            id: 0,
            content: Default::default(),
            regions: RelationalPersistentRegionSet::initial(0, BTreeMap::new(), &issuer)
                .expect("empty test root has reachability capacity"),
            entity_slot_count: 0,
            relation_slot_count: 0,
            content_accumulator: [0; 32],
            output_publication_positions: Default::default(),
            execution_contracts: Default::default(),
            schema_authority: RelationalBranchRootSchemaAuthority::empty(),
            committed: None,
            _memory: None,
            managed_source_image: std::sync::OnceLock::new(),
        })
    }

    pub(crate) fn empty_with_schema(
        registry: &crate::schema::data::RelationalSchemaRegistry,
        descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
    ) -> Arc<Self> {
        let issuer = RelationalBranchRootIdentityIssuer::default();
        let expected = registry.authority_snapshot();
        let schema_authority_id = issuer
            .issue_schema_authority_id()
            .expect(/* Proof: Each empty root creates a fresh issuer at its initial frontier; these first schema and empty-reachability allocations cannot exhaust it. */ "empty root has schema authority capacity");
        let schema = RelationalBranchRootSchemaAuthority::capture(
            schema_authority_id, registry, &expected, descriptor_semantics_version, None,
        ).expect(/* Proof: the authority is captured from this same validated registry and its own snapshot. */ "empty schema matches registry");
        Self::empty_with_schema_authority(schema)
    }

    pub(crate) fn empty_with_schema_authority(
        schema: Arc<RelationalBranchRootSchemaAuthority>,
    ) -> Arc<Self> {
        let issuer = RelationalBranchRootIdentityIssuer::default();
        Arc::new(Self {
            id: 0,
            content: Default::default(),
            regions: RelationalPersistentRegionSet::initial(0, BTreeMap::new(), &issuer)
                .expect(/* Proof: Each empty root creates a fresh issuer at its initial frontier; these first schema and empty-reachability allocations cannot exhaust it. */ "empty root has reachability capacity"),
            entity_slot_count: 0,
            relation_slot_count: 0,
            content_accumulator: [0; 32],
            output_publication_positions: Default::default(),
            execution_contracts: Default::default(),
            schema_authority: schema,
            committed: None,
            _memory: None,
            managed_source_image: std::sync::OnceLock::new(),
        })
    }
}
