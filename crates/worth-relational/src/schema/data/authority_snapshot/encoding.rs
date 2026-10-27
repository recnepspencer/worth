//! Optional authority tails preserve positional and named legacy serializers.
use super::SchemaAuthoritySnapshot;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

impl Serialize for SchemaAuthoritySnapshot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let domains = !self.projected_member_domains.is_empty();
        let derived = domains || !self.derived_relations.is_empty();
        let mut output = serializer.serialize_struct(
            "SchemaAuthoritySnapshot",
            4 + usize::from(derived) + usize::from(domains),
        )?;
        output.serialize_field("primary_schema_id", &self.primary_schema_id)?;
        output.serialize_field("primary_schema_version_id", &self.primary_schema_version_id)?;
        output.serialize_field("entity_kinds", &self.entity_kinds)?;
        output.serialize_field("relation_kinds", &self.relation_kinds)?;
        // A nonempty later tail must not slide into a skipped earlier slot in
        // compact struct formats. Empty physical authorities still emit four.
        if derived {
            output.serialize_field("derived_relations", &self.derived_relations)?;
        }
        if domains {
            output.serialize_field("projected_member_domains", &self.projected_member_domains)?;
        }
        output.end()
    }
}
