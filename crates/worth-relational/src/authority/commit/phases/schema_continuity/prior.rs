//! Previous schema facts come from the selected root's actual evidence.
pub(super) struct SchemaContinuityPrior<'a> {
    pub(super) schema_version: crate::schema::data::SchemaVersionId,
    pub(super) descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
    pub(super) authority: &'a crate::branch::RelationalBranchRootSchemaAuthority,
}
impl<'a> SchemaContinuityPrior<'a> {
    pub(super) fn from_root(root: &'a crate::branch::RelationalBranchRoot) -> Option<Self> {
        let evidence = root.commit_evidence()?;
        let authority = root.schema_authority();
        Some(Self {
            schema_version: evidence.schema_version(),
            descriptor_semantics_version: authority.descriptor_semantics_version(),
            authority,
        })
    }
}
