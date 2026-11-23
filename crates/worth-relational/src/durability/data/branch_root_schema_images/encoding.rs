//! Required content authority fixes every preceding durable tail slot.
use super::DurableBranchRootSchemaImage;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

impl Serialize for DurableBranchRootSchemaImage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let has_naming =
            !self.0.persistent_name_schemes.is_empty() || !self.0.kind_name_schemes.is_empty();
        let field_count = if has_naming { 12 } else { 10 };
        let mut output =
            serializer.serialize_struct("DurableBranchRootSchemaImage", field_count)?;
        output.serialize_field("format_version", &self.0.format_version)?;
        output.serialize_field("carrier_digest", &self.0.carrier_digest)?;
        output.serialize_field("schema_root", &self.0.schema_root)?;
        output.serialize_field(
            "descriptor_semantics_version",
            &self.0.descriptor_semantics_version,
        )?;
        output.serialize_field("entity_registrations", &self.0.entity_registrations)?;
        output.serialize_field("relation_registrations", &self.0.relation_registrations)?;
        output.serialize_field(
            "retained_aspect_contracts",
            &self.0.retained_aspect_contracts,
        )?;
        output.serialize_field("derived_relations", &self.0.derived_relations)?;
        output.serialize_field("projected_member_domains", &self.0.projected_member_domains)?;
        output.serialize_field("content_formats", &self.0.content_formats)?;
        if has_naming {
            output.serialize_field("persistent_name_schemes", &self.0.persistent_name_schemes)?;
            output.serialize_field("kind_name_schemes", &self.0.kind_name_schemes)?;
        }
        output.end()
    }
}
