//! Project one registry into its canonical durable schema authority.
use super::*;

mod projection_count;

impl RelationalSchemaRegistry {
    pub fn authority_snapshot(&self) -> SchemaAuthoritySnapshot {
        projection_count::record(
            self.entity_kinds.len()
                + self.relation_kinds.len()
                + self.derived_relations.len()
                + self.projected_member_domains.len(),
        );
        let (primary_schema_id, primary_schema_version_id) = self
            .authoritative_schema_basis()
            .ok()
            .flatten()
            .map(|(schema_id, schema_version_id)| (Some(schema_id), Some(schema_version_id)))
            .unwrap_or((None, None));

        let entity_kinds = self
            .entity_kinds
            .iter()
            .map(|(kind_id, registration)| SchemaAuthorityKindSnapshot {
                kind_id: *kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
            })
            .collect();
        let relation_kinds = self
            .relation_kinds
            .iter()
            .map(|(kind_id, registration)| SchemaAuthorityRelationSnapshot {
                kind_id: *kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
                relation_integrity_plan_revision: derive_relation_integrity_plan_revision(
                    &registration.relation_integrity.endpoint_kind_contracts,
                    &registration.relation_integrity.cardinality_contracts,
                    &registration.relation_integrity.uniqueness_contracts,
                    &registration.relation_integrity.symmetry_contracts,
                    &registration
                        .relation_integrity
                        .endpoint_deletion_integrity_contracts,
                    &registration.relation_integrity.acyclicity_contracts,
                    &registration
                        .relation_integrity
                        .partition_isolation_contracts,
                    &registration
                        .relation_integrity
                        .connectivity_minimum_contracts,
                    registration.relation_integrity.aggregate_identity.as_ref(),
                    registration
                        .relation_integrity
                        .current_publication_storage
                        .as_ref(),
                ),
                cross_context_policy: registration.cross_context_policy,
                cascade_delete_policy: registration.cascade_delete_policy,
            })
            .collect();

        crate::schema::data::SchemaAuthoritySnapshotParts {
            content_formats: self.content_formats.snapshot(),
            primary_schema_id,
            primary_schema_version_id,
            entity_kinds,
            relation_kinds,
            derived_relations: self.derived_parts(),
            projected_member_domains: self.projected_domain_parts(),
        }
        .into()
    }

    pub fn authority_digest_bytes(&self) -> [u8; 32] {
        schema_authority_snapshot_digest_bytes(&self.authority_snapshot())
    }
}
