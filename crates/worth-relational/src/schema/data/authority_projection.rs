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
        self.snapshot_for_compilation(crate::schema::SchemaCompilationControl::Configuration)
            .expect("configuration descriptor assembly has no request refusal")
    }

    pub fn authority_digest_bytes(&self) -> [u8; 32] {
        schema_authority_snapshot_digest_bytes(&self.authority_snapshot())
    }
}
