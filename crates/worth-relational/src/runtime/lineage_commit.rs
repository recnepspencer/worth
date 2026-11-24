//! Bind the operation's selected schema, names and identity allocator to lineage preparation.
use crate::authority::commit::lineage_preparation::LineagePreparationAuthority;
use crate::capabilities::SchemaSource;
use crate::runtime::RelationalPreparationRuntime;

impl RelationalPreparationRuntime {
    pub(crate) fn lineage_preparation_authority(&self) -> LineagePreparationAuthority<'_> {
        LineagePreparationAuthority {
            schema: self.schema_registry(),
            naming: &self.lineage_naming_basis,
            identities: &self.lineage_identity,
            instrumentation: &self.services.instrumentation,
        }
    }
}
