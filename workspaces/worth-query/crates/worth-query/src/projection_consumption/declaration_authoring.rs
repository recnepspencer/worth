use crate::authorized_projection::AuthorizedProjectionArtifact;
use crate::canonicalization::CanonicalResultShapeArtifact;
use crate::query_context::QueryContextExecutionArtifact;
use crate::runtime::{
    WorthQueryDerivedArtifactBinding, WorthQueryLiveArtifactBinding, WorthQueryReadReceipt,
    WorthQueryWriteReceipt,
};
use worth_runtime_bridge::facade::{
    BridgeGroupedTruthViewArtifact, BridgeMaterializedRowSetArtifact,
};
use worth_runtime_bridge::facade::{
    RelationalAuthoritativeRowSetArtifact, RelationalGroupedProjectionArtifact,
};

use super::declaration::{
    declare_projection_consumption, ProjectionConsumptionBindingContext,
    ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError,
};
use super::facts::{ProjectMaterializedFacts, ProjectionFactFieldPath};
use super::source::ProjectionConsumptionSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionAuthoringSurface {
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
}

impl ProjectionConsumptionAuthoringSurface {
    pub fn from_read_receipt(
        receipt: &WorthQueryReadReceipt,
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_read_receipt(receipt, result_shape),
            binding: ProjectionConsumptionBindingContext::from_authorized_projection(
                result_shape,
                authorized_projection,
            ),
        }
    }

    pub fn from_write_receipt(
        receipt: &WorthQueryWriteReceipt,
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_write_receipt(receipt),
            binding: ProjectionConsumptionBindingContext::from_result_shape_digest(
                result_shape_digest,
                authorized_projection,
            ),
        }
    }

    pub fn from_query_context_execution(
        execution: &QueryContextExecutionArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_query_context_execution(execution),
            binding: ProjectionConsumptionBindingContext::from_result_shape_digest(
                execution.result_shape_digest(),
                authorized_projection,
            ),
        }
    }

    pub fn from_relational_row_set(
        row_set: &RelationalAuthoritativeRowSetArtifact,
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_relational_row_set(row_set),
            binding: ProjectionConsumptionBindingContext::from_result_shape_digest(
                result_shape_digest,
                authorized_projection,
            ),
        }
    }

    pub fn from_relational_grouped_projection(
        grouped_projection: &RelationalGroupedProjectionArtifact,
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_relational_grouped_projection(
                grouped_projection,
            ),
            binding: ProjectionConsumptionBindingContext::from_result_shape_digest(
                result_shape_digest,
                authorized_projection,
            ),
        }
    }

    pub fn from_bridge_truth_view_row_set(
        row_set: &BridgeMaterializedRowSetArtifact,
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_bridge_truth_view_row_set(row_set),
            binding: ProjectionConsumptionBindingContext::from_result_shape_digest(
                result_shape_digest,
                authorized_projection,
            ),
        }
    }

    pub fn from_bridge_grouped_truth_view(
        grouped_truth_view: &BridgeGroupedTruthViewArtifact,
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_bridge_grouped_truth_view(grouped_truth_view),
            binding: ProjectionConsumptionBindingContext::from_result_shape_digest(
                result_shape_digest,
                authorized_projection,
            ),
        }
    }

    pub fn from_retained_derived_artifact_binding(
        binding: &WorthQueryDerivedArtifactBinding,
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_retained_derived_artifact_binding(binding),
            binding: ProjectionConsumptionBindingContext::from_authorized_projection(
                result_shape,
                authorized_projection,
            ),
        }
    }

    pub fn from_live_artifact_binding(
        binding: &WorthQueryLiveArtifactBinding,
        result_shape_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        authorized_projection: &AuthorizedProjectionArtifact,
    ) -> Self {
        Self {
            source: ProjectionConsumptionSource::from_live_artifact_binding(binding),
            binding: ProjectionConsumptionBindingContext::from_result_shape_identity(
                result_shape_identity,
                authorized_projection,
            ),
        }
    }

    pub fn source(&self) -> &ProjectionConsumptionSource {
        &self.source
    }

    pub fn binding(&self) -> &ProjectionConsumptionBindingContext {
        &self.binding
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionDeclarationBuilder {
    surface: ProjectionConsumptionAuthoringSurface,
    requested: ProjectMaterializedFacts,
}

impl ProjectionConsumptionDeclarationBuilder {
    pub fn entity_identities(mut self) -> Self {
        self.requested = self.requested.entity_identities();
        self
    }

    pub fn view_local_identities(mut self) -> Self {
        self.requested = self.requested.view_local_identities();
        self
    }

    pub fn target_identity(mut self) -> Self {
        self.requested = self.requested.target_identity();
        self
    }

    pub fn source_references(mut self) -> Self {
        self.requested = self.requested.source_references();
        self
    }

    pub fn effect_continuity_facts(mut self) -> Self {
        self.requested = self.requested.effect_continuity_facts();
        self
    }

    pub fn memberships(mut self) -> Self {
        self.requested = self.requested.memberships();
        self
    }

    pub fn relation_endpoints(mut self) -> Self {
        self.requested = self.requested.relation_endpoints();
        self
    }

    pub fn display_field_path(mut self, field: ProjectionFactFieldPath) -> Self {
        self.requested = self.requested.display_field_path(field);
        self
    }

    pub fn derived_field_path(mut self, field: ProjectionFactFieldPath) -> Self {
        self.requested = self.requested.derived_field_path(field);
        self
    }

    pub fn build(
        self,
    ) -> Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError> {
        declare_projection_consumption(self.surface.source, self.surface.binding, self.requested)
    }
}

impl ProjectMaterializedFacts {
    pub fn source(
        self,
        surface: ProjectionConsumptionAuthoringSurface,
    ) -> ProjectionConsumptionDeclarationBuilder {
        ProjectionConsumptionDeclarationBuilder {
            surface,
            requested: self,
        }
    }
}

impl WorthQueryReadReceipt {
    pub(crate) fn declare_projection_fact_consumption(
        &self,
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError> {
        requested
            .source(ProjectionConsumptionAuthoringSurface::from_read_receipt(
                self,
                result_shape,
                authorized_projection,
            ))
            .build()
    }
}

impl WorthQueryWriteReceipt {
    pub(crate) fn declare_projection_fact_consumption(
        &self,
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError> {
        requested
            .source(ProjectionConsumptionAuthoringSurface::from_write_receipt(
                self,
                result_shape_digest,
                authorized_projection,
            ))
            .build()
    }
}

impl QueryContextExecutionArtifact {
    pub(crate) fn declare_projection_fact_consumption(
        &self,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError> {
        requested
            .source(
                ProjectionConsumptionAuthoringSurface::from_query_context_execution(
                    self,
                    authorized_projection,
                ),
            )
            .build()
    }
}

impl WorthQueryDerivedArtifactBinding {
    pub(crate) fn declare_projection_fact_consumption(
        &self,
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError> {
        requested
            .source(
                ProjectionConsumptionAuthoringSurface::from_retained_derived_artifact_binding(
                    self,
                    result_shape,
                    authorized_projection,
                ),
            )
            .build()
    }
}

impl WorthQueryLiveArtifactBinding {
    pub(crate) fn declare_projection_fact_consumption(
        &self,
        result_shape_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError> {
        requested
            .source(
                ProjectionConsumptionAuthoringSurface::from_live_artifact_binding(
                    self,
                    result_shape_identity,
                    authorized_projection,
                ),
            )
            .build()
    }
}
