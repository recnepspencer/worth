//! The admitted schema owner issues continuity DATA from its accepted projection.
use super::*;
impl RelationalBranchRootSchemaAuthority {
    pub(crate) fn continuity_input(
        &self,
        basis: crate::schema::data::DescriptorCanonicalBasisVersion,
    ) -> crate::schema::SchemaContinuityAuthorityInput {
        crate::schema::SchemaContinuityAuthorityInput::from_carried_registry(
            self.registry_arc(),
            self.continuity_authority.clone(),
            self.descriptor_semantics_version,
            basis,
        )
    }
}
