//! Previous schema facts come from the selected root's actual evidence.
pub(super) struct SchemaContinuityPrior<'a> {
    pub(super) schema_version: worth_relational_contracts::internal_port::schema::data::SchemaVersionId,
    pub(super) descriptor_semantics_version: worth_relational_contracts::internal_port::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
    pub(super) authority: &'a worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
}
impl<'a> SchemaContinuityPrior<'a> {
    pub(super) fn from_root(
        root: &'a worth_relational_branch::internal_port::branch::root::RelationalBranchRoot,
    ) -> Option<Self> {
        let evidence = root.commit_evidence()?;
        let authority = root.schema_authority();
        Some(Self {
            schema_version: evidence.schema_version(),
            descriptor_semantics_version: authority.descriptor_semantics_version(),
            authority,
        })
    }
}
