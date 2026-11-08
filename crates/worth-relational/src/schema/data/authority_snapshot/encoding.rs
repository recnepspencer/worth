//! Required content authority fixes every preceding positional tail slot.
use super::SchemaAuthoritySnapshotParts;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

impl Serialize for SchemaAuthoritySnapshotParts {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut output = serializer.serialize_struct("SchemaAuthoritySnapshotParts", 7)?;
        output.serialize_field("primary_schema_id", &self.primary_schema_id)?;
        output.serialize_field("primary_schema_version_id", &self.primary_schema_version_id)?;
        output.serialize_field("entity_kinds", &self.entity_kinds)?;
        output.serialize_field("relation_kinds", &self.relation_kinds)?;
        output.serialize_field("derived_relations", &self.derived_relations)?;
        output.serialize_field("projected_member_domains", &self.projected_member_domains)?;
        output.serialize_field("content_formats", &self.content_formats)?;
        output.end()
    }
}
