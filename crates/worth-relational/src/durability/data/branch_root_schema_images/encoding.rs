//! Conditional durable tails keep the old carrier struct shape byte-exact.
use super::DurableBranchRootSchemaImage;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

impl Serialize for DurableBranchRootSchemaImage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let domains = !self.projected_member_domains.is_empty();
        let derived = domains || !self.derived_relations.is_empty();
        let mut output = serializer.serialize_struct(
            "DurableBranchRootSchemaImage",
            7 + usize::from(derived) + usize::from(domains),
        )?;
        output.serialize_field("format_version", &self.format_version)?;
        output.serialize_field("carrier_digest", &self.carrier_digest)?;
        output.serialize_field("schema_root", &self.schema_root)?;
        output.serialize_field(
            "descriptor_semantics_version",
            &self.descriptor_semantics_version,
        )?;
        output.serialize_field("entity_registrations", &self.entity_registrations)?;
        output.serialize_field("relation_registrations", &self.relation_registrations)?;
        output.serialize_field("retained_aspect_contracts", &self.retained_aspect_contracts)?;
        if derived {
            output.serialize_field("derived_relations", &self.derived_relations)?;
        }
        if domains {
            output.serialize_field("projected_member_domains", &self.projected_member_domains)?;
        }
        output.end()
    }
}
