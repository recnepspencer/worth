use crate::authorized_projection::{AuthorizedProjectionArtifact, AuthorizedProjectionFieldPath};
use crate::canonicalization::CanonicalResultShapeArtifact;
#[cfg(test)]
use worth_foundational::facade::{AspectKey, FieldKey};

use super::facts::ProjectMaterializedFacts;
use super::identity::compose_declaration_digest;
use super::source::ProjectionConsumptionSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionBindingContext {
    result_shape_digest: String,
    authorized_projection_query_digest: String,
    authorized_projection_result_shape_digest: String,
    authorized_projection_identity: String,
    narrowed_result_shape_digest: String,
    policy_digest: String,
    tenant_schema_basis_digest: String,
    authorized_visible_fields: Vec<AuthorizedProjectionFieldPath>,
}

impl ProjectionConsumptionBindingContext {
    pub fn from_authorized_projection(
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self::from_result_shape_digest(result_shape.digest().as_str(), authorized_projection)
    }

    pub fn from_result_shape_digest(
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            result_shape_digest: result_shape_digest.to_string(),
            authorized_projection_query_digest: authorized_projection.query_digest().to_string(),
            authorized_projection_result_shape_digest: authorized_projection
                .result_shape_digest()
                .to_string(),
            authorized_projection_identity: authorized_projection.identity().as_str().to_string(),
            narrowed_result_shape_digest: authorized_projection
                .narrowed_result_shape_digest()
                .to_string(),
            policy_digest: authorized_projection.policy_digest().to_string(),
            tenant_schema_basis_digest: authorized_projection
                .tenant_schema_basis_digest()
                .to_string(),
            authorized_visible_fields: authorized_projection.visible_field_paths().to_vec(),
        }
    }

    pub fn from_result_shape_identity(
        result_shape_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self::from_result_shape_digest(result_shape_identity.as_str(), authorized_projection)
    }

    pub fn from_projection_metadata(
        result_shape_digest: impl Into<String>,
        authorized_projection_query_digest: impl Into<String>,
        authorized_projection_result_shape_digest: impl Into<String>,
        authorized_projection_identity: impl Into<String>,
        narrowed_result_shape_digest: impl Into<String>,
        policy_digest: impl Into<String>,
        tenant_schema_basis_digest: impl Into<String>,
        authorized_visible_fields: Vec<AuthorizedProjectionFieldPath>,
    ) -> Self {
        Self {
            result_shape_digest: result_shape_digest.into(),
            authorized_projection_query_digest: authorized_projection_query_digest.into(),
            authorized_projection_result_shape_digest: authorized_projection_result_shape_digest
                .into(),
            authorized_projection_identity: authorized_projection_identity.into(),
            narrowed_result_shape_digest: narrowed_result_shape_digest.into(),
            policy_digest: policy_digest.into(),
            tenant_schema_basis_digest: tenant_schema_basis_digest.into(),
            authorized_visible_fields,
        }
    }

    pub fn result_shape_digest(&self) -> &str {
        &self.result_shape_digest
    }

    pub fn authorized_projection_identity(&self) -> &str {
        &self.authorized_projection_identity
    }

    pub fn authorized_projection_query_digest(&self) -> &str {
        &self.authorized_projection_query_digest
    }

    pub fn authorized_projection_result_shape_digest(&self) -> &str {
        &self.authorized_projection_result_shape_digest
    }

    pub fn narrowed_result_shape_digest(&self) -> &str {
        &self.narrowed_result_shape_digest
    }

    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    pub fn tenant_schema_basis_digest(&self) -> &str {
        &self.tenant_schema_basis_digest
    }

    pub fn authorized_visible_field_paths(&self) -> &[AuthorizedProjectionFieldPath] {
        &self.authorized_visible_fields
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        result_shape_digest: impl Into<String>,
        authorized_projection_identity: impl Into<String>,
        authorized_visible_fields: Vec<AuthorizedProjectionFieldPath>,
    ) -> Self {
        Self {
            result_shape_digest: result_shape_digest.into(),
            authorized_projection_query_digest: "query:test".to_string(),
            authorized_projection_result_shape_digest: "result-shape:test".to_string(),
            authorized_projection_identity: authorized_projection_identity.into(),
            narrowed_result_shape_digest: "narrowed-result-shape:test".to_string(),
            policy_digest: "policy:test".to_string(),
            tenant_schema_basis_digest: "tenant-schema:test".to_string(),
            authorized_visible_fields,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_only_with_projection_metadata(
        result_shape_digest: impl Into<String>,
        authorized_projection_query_digest: impl Into<String>,
        authorized_projection_result_shape_digest: impl Into<String>,
        authorized_projection_identity: impl Into<String>,
        narrowed_result_shape_digest: impl Into<String>,
        policy_digest: impl Into<String>,
        tenant_schema_basis_digest: impl Into<String>,
        authorized_visible_fields: Vec<AuthorizedProjectionFieldPath>,
    ) -> Self {
        let result_shape_digest = result_shape_digest.into();
        Self {
            result_shape_digest: result_shape_digest.clone(),
            authorized_projection_query_digest: authorized_projection_query_digest.into(),
            authorized_projection_result_shape_digest: authorized_projection_result_shape_digest
                .into(),
            authorized_projection_identity: authorized_projection_identity.into(),
            narrowed_result_shape_digest: narrowed_result_shape_digest.into(),
            policy_digest: policy_digest.into(),
            tenant_schema_basis_digest: tenant_schema_basis_digest.into(),
            authorized_visible_fields,
        }
    }

    pub(crate) fn intent_admission_certification_binding(
        result_shape_digest: impl Into<String>,
        authorized_projection_query_digest: impl Into<String>,
        authorized_projection_result_shape_digest: impl Into<String>,
        authorized_projection_identity: impl Into<String>,
        narrowed_result_shape_digest: impl Into<String>,
        policy_digest: impl Into<String>,
        tenant_schema_basis_digest: impl Into<String>,
        authorized_visible_fields: Vec<AuthorizedProjectionFieldPath>,
    ) -> Self {
        let result_shape_digest = result_shape_digest.into();
        Self {
            result_shape_digest: result_shape_digest.clone(),
            authorized_projection_query_digest: authorized_projection_query_digest.into(),
            authorized_projection_result_shape_digest: authorized_projection_result_shape_digest
                .into(),
            authorized_projection_identity: authorized_projection_identity.into(),
            narrowed_result_shape_digest: narrowed_result_shape_digest.into(),
            policy_digest: policy_digest.into(),
            tenant_schema_basis_digest: tenant_schema_basis_digest.into(),
            authorized_visible_fields,
        }
    }
}

#[cfg(test)]
pub(crate) fn test_authorized_field_paths(fields: &[&str]) -> Vec<AuthorizedProjectionFieldPath> {
    fields
        .iter()
        .map(|field| {
            authorized_projection_field_path_from_test_boundary(field)
                .expect("test authorized projection field path should be foundational")
        })
        .collect()
}

#[cfg(test)]
fn authorized_projection_field_path_from_test_boundary(
    field: &str,
) -> Result<AuthorizedProjectionFieldPath, String> {
    let Some((aspect, field)) = field.split_once('.') else {
        let aspect_key = AspectKey::new(field.to_string())
            .ok_or_else(|| format!("`{field}` is not a foundational aspect key"))?;
        return Ok(AuthorizedProjectionFieldPath::from_native_aspect_key(
            aspect_key,
        ));
    };
    let aspect_key = AspectKey::new(aspect.to_string())
        .ok_or_else(|| format!("`{aspect}` is not a foundational aspect key"))?;
    let field_key = FieldKey::new(field.to_string())
        .ok_or_else(|| format!("`{field}` is not a foundational field key"))?;
    Ok(AuthorizedProjectionFieldPath::from_native_keys(
        aspect_key, field_key,
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionDeclaration {
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
    requested: ProjectMaterializedFacts,
    declaration_digest: String,
}

impl ProjectionConsumptionDeclaration {
    pub fn source(&self) -> &ProjectionConsumptionSource {
        &self.source
    }

    pub fn binding(&self) -> &ProjectionConsumptionBindingContext {
        &self.binding
    }

    pub fn requested(&self) -> &ProjectMaterializedFacts {
        &self.requested
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionDeclarationError {
    NoRequestedFacts,
    SourceAuthorizedProjectionQueryMismatch {
        source_query_digest: String,
        authorized_projection_query_digest: String,
    },
    BindingAuthorizedProjectionResultShapeMismatch {
        binding_result_shape_digest: String,
        authorized_projection_result_shape_digest: String,
    },
    SourceBindingResultShapeMismatch {
        source_result_shape_digest: String,
        binding_result_shape_digest: String,
    },
}

pub fn declare_projection_consumption(
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
    requested: ProjectMaterializedFacts,
) -> Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError> {
    if requested.requested_count() == 0 {
        return Err(ProjectionConsumptionDeclarationError::NoRequestedFacts);
    }
    if let Some(source_query_digest) = source.query_digest()
        && source_query_digest != binding.authorized_projection_query_digest()
    {
        return Err(
            ProjectionConsumptionDeclarationError::SourceAuthorizedProjectionQueryMismatch {
                source_query_digest: source_query_digest.to_string(),
                authorized_projection_query_digest: binding
                    .authorized_projection_query_digest()
                    .to_string(),
            },
        );
    }
    if binding.result_shape_digest() != binding.authorized_projection_result_shape_digest() {
        return Err(
            ProjectionConsumptionDeclarationError::BindingAuthorizedProjectionResultShapeMismatch {
                binding_result_shape_digest: binding.result_shape_digest().to_string(),
                authorized_projection_result_shape_digest: binding
                    .authorized_projection_result_shape_digest()
                    .to_string(),
            },
        );
    }
    if let Some(source_result_shape_digest) = source.result_shape_digest()
        && source_result_shape_digest != binding.result_shape_digest()
    {
        return Err(
            ProjectionConsumptionDeclarationError::SourceBindingResultShapeMismatch {
                source_result_shape_digest: source_result_shape_digest.to_string(),
                binding_result_shape_digest: binding.result_shape_digest().to_string(),
            },
        );
    }
    let declaration_digest = compose_declaration_digest(&source, &binding, &requested);
    Ok(ProjectionConsumptionDeclaration {
        source,
        binding,
        requested,
        declaration_digest,
    })
}
