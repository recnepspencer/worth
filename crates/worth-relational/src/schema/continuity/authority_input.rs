use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion, SchemaAuthoritySnapshot, SchemaId,
    SchemaVersionId,
};

/// Schema authority selected by the owner for one continuity derivation.
///
/// Ordinary commits derive this from their admitted root. Declared schema
/// transitions use the live target, while recovery may carry an admitted
/// historical snapshot so replay does not reinterpret an older commit through
/// the runtime's final schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchemaContinuityAuthorityInput {
    target_schema_registry: Option<std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>>,
    target_schema_version: SchemaVersionId,
    target_schema_authority: SchemaAuthoritySnapshot,
    target_schema_basis: Option<(SchemaId, SchemaVersionId)>,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
}

impl SchemaContinuityAuthorityInput {
    pub(crate) fn from_root(
        root: &crate::branch::RelationalBranchRootSchemaAuthority,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    ) -> Self {
        let mut input = Self::new(
            root.schema_version(),
            root.snapshot().clone(),
            root.descriptor_semantics_version(),
            descriptor_canonical_basis_version,
        );
        input.target_schema_registry = Some(root.registry_arc());
        input
    }

    pub(crate) fn from_registry(
        target_schema_registry: crate::schema::data::RelationalSchemaRegistry,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    ) -> Self {
        Self::from_shared_registry(
            std::sync::Arc::new(target_schema_registry),
            descriptor_semantics_version,
            descriptor_canonical_basis_version,
        )
    }

    pub(crate) fn from_shared_registry(
        target_schema_registry: std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    ) -> Self {
        let target_schema_authority = target_schema_registry.authority_snapshot();
        let target_schema_version = target_schema_authority
            .primary_schema_version_id
            .unwrap_or(SchemaVersionId(0));
        let mut input = Self::new(
            target_schema_version,
            target_schema_authority,
            descriptor_semantics_version,
            descriptor_canonical_basis_version,
        );
        input.target_schema_registry = Some(target_schema_registry);
        input
    }

    pub(crate) fn new(
        target_schema_version: SchemaVersionId,
        target_schema_authority: SchemaAuthoritySnapshot,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    ) -> Self {
        let target_schema_basis = target_schema_authority
            .primary_schema_id
            .clone()
            .zip(target_schema_authority.primary_schema_version_id);
        Self {
            target_schema_registry: None,
            target_schema_version,
            target_schema_authority,
            target_schema_basis,
            descriptor_semantics_version,
            descriptor_canonical_basis_version,
        }
    }

    pub(crate) fn target_schema_registry(
        &self,
    ) -> Option<&std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>> {
        self.target_schema_registry.as_ref()
    }

    pub(crate) const fn target_schema_version(&self) -> SchemaVersionId {
        self.target_schema_version
    }

    pub(crate) fn target_schema_authority(&self) -> &SchemaAuthoritySnapshot {
        &self.target_schema_authority
    }

    pub(crate) fn target_schema_basis(&self) -> Option<(SchemaId, SchemaVersionId)> {
        self.target_schema_basis.clone()
    }

    pub(crate) const fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.descriptor_semantics_version
    }

    pub(crate) const fn descriptor_canonical_basis_version(
        &self,
    ) -> DescriptorCanonicalBasisVersion {
        self.descriptor_canonical_basis_version
    }
}
