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
pub struct SchemaContinuityAuthorityInput {
    target_schema_registry:
        Option<crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry>,
    target_schema_version: SchemaVersionId,
    target_schema_authority: SchemaAuthoritySnapshot,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
}

impl SchemaContinuityAuthorityInput {
    pub fn from_carried_registry(
        registry: crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry,
        authority: SchemaAuthoritySnapshot,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    ) -> Self {
        let version = authority
            .primary_schema_version_id
            .unwrap_or(SchemaVersionId(0));
        let mut input = Self::new(
            version,
            authority,
            descriptor_semantics_version,
            descriptor_canonical_basis_version,
        );
        input.target_schema_registry = Some(registry);
        input
    }

    pub fn copy_registry_for_request(
        registry: &crate::schema::data::RelationalSchemaRegistry,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::branch::root::RelationalBranchRootCaptureDenial> {
        let registry =
            crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry::copy_for_request(registry, request)?;
        let authority = registry
            .snapshot_for_compilation(crate::schema::SchemaCompilationControl::Request(*request))
            .map_err(crate::branch::root::RelationalBranchRootCaptureDenial::Request)?;
        Ok(Self::from_carried_registry(
            registry,
            authority,
            descriptor_semantics_version,
            descriptor_canonical_basis_version,
        ))
    }

    pub fn from_registry(
        target_schema_registry: crate::schema::data::RelationalSchemaRegistry,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    ) -> Self {
        Self::from_shared_registry(
            crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry::from_configuration(
                std::sync::Arc::new(target_schema_registry),
            ),
            descriptor_semantics_version,
            descriptor_canonical_basis_version,
        )
    }

    pub fn from_shared_registry(
        target_schema_registry: crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry,
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

    pub fn new(
        target_schema_version: SchemaVersionId,
        target_schema_authority: SchemaAuthoritySnapshot,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
    ) -> Self {
        Self {
            target_schema_registry: None,
            target_schema_version,
            target_schema_authority,
            descriptor_semantics_version,
            descriptor_canonical_basis_version,
        }
    }

    /// Recovery replay carries the recorded result schema's own registry with
    /// the snapshot its commit recorded, so the new root is captured under it.
    pub fn carrying_recorded_registry(
        mut self,
        registry: crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry,
    ) -> Self {
        self.target_schema_registry = Some(registry);
        self
    }

    pub fn target_schema_registry(
        &self,
    ) -> Option<&crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry>
    {
        self.target_schema_registry.as_ref()
    }

    pub const fn target_schema_version(&self) -> SchemaVersionId {
        self.target_schema_version
    }

    pub fn target_schema_authority(&self) -> &SchemaAuthoritySnapshot {
        &self.target_schema_authority
    }

    pub fn target_schema_basis(&self) -> Option<(SchemaId, SchemaVersionId)> {
        self.target_schema_authority
            .primary_schema_id
            .clone()
            .zip(self.target_schema_authority.primary_schema_version_id)
    }

    pub const fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.descriptor_semantics_version
    }

    pub const fn descriptor_canonical_basis_version(&self) -> DescriptorCanonicalBasisVersion {
        self.descriptor_canonical_basis_version
    }
}
