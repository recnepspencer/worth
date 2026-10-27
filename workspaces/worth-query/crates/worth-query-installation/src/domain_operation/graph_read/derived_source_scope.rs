//! Installed association for a real declared derived source and full schema basis.
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

/// Only installation issues this scope after comparing the real Member43 closure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationDerivedSourceReadScope {
    schema: ApplicationSchemaBindingIdentity,
    relation: String,
}
impl WorthQueryOperationDerivedSourceReadScope {
    pub(crate) fn new(schema: ApplicationSchemaBindingIdentity, relation: String) -> Self {
        Self { schema, relation }
    }
    pub fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub fn relation(&self) -> &str {
        &self.relation
    }
    /// Exact installed association, never a Native source observation.
    pub fn association_identity(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(b"worth-query:installed-derived-source:v1");
        digest.update(self.schema.runtime_ordinal().to_be_bytes());
        digest.update(self.schema.generation().to_be_bytes());
        digest.update(self.schema.package_identity().bytes());
        digest.update(self.schema.schema_identity().bytes());
        digest.update((self.relation.len() as u64).to_be_bytes());
        digest.update(self.relation.as_bytes());
        digest.finalize().into()
    }
}
