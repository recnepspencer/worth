use crate::identity::data::{KindId, VersionId};

use super::contracts::{
    admit_declared_projection_aspects, ProjectionAspectScope, ProjectionRecordDomain,
};
use super::projection_records::{EntityRecordProjection, RelationRecordProjection};

pub struct SelectedProjectionSchema<'a> {
    pub registry: &'a crate::schema::data::RelationalSchemaRegistry,
    pub plans: &'a crate::schema::data::AspectContractPlanCatalog,
    pub version: crate::schema::data::SchemaVersionId,
    pub authority_digest: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct VisibilityProjectionView {
    pub(super) runtime_identity: u64,
    pub(super) current_version: VersionId,
    pub(super) instrumentation:
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    pub(super) symbols:
        crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    // Selected roots already retain their exact schema. Only empty history needs a fallback.
    pub(super) empty_history_schema:
        Option<std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>>,
    pub(super) resident:
        Option<crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis>,
    pub(super) basis: crate::visibility::snapshot_states::SnapshotStateBasis,
}

impl VisibilityProjectionView {
    pub const fn version_id(&self) -> VersionId {
        self.basis.version_id()
    }

    pub(crate) fn snapshot_basis(&self) -> &crate::visibility::snapshot_states::SnapshotStateBasis {
        &self.basis
    }

    pub fn is_exact_basis(&self) -> bool {
        matches!(
            self.basis,
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(_)
        )
    }

    pub(crate) fn selected_branch_id(&self) -> &crate::history::data::BranchId {
        self.basis.branch_id()
    }

    pub(crate) fn shares_runtime_with(&self, other: &Self) -> bool {
        self.is_from_runtime(other.runtime_identity)
    }





    pub(super) fn reader(&self) -> crate::visibility::materialization::read_records::reader::projection_record_reader::ProjectionRecordReader<'_>{
        crate::visibility::materialization::read_records::reader::projection_record_reader::ProjectionRecordReader::new(
            self.current_version,
            &self.instrumentation,
        )
    }

    pub(super) fn admit_entity_projection_contract<T: EntityRecordProjection>(
        &self,
    ) -> Result<ProjectionAspectScope, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        let projection_scope = T::projection_scope();
        admit_declared_projection_aspects(
            &projection_scope,
            self.entity_aspect_plan(T::KIND),
            ProjectionRecordDomain::Entity,
            T::KIND,
        )
        .map_err(
            crate::execution::denial_cause::RelationalExecutionDenialCause::ProjectionContract,
        )?;
        Ok(projection_scope)
    }

    pub(super) fn admit_relation_projection_contract<T: RelationRecordProjection>(
        &self,
    ) -> Result<ProjectionAspectScope, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        let projection_scope = T::projection_scope();
        admit_declared_projection_aspects(
            &projection_scope,
            self.relation_aspect_plan(T::KIND),
            ProjectionRecordDomain::Relation,
            T::KIND,
        )
        .map_err(
            crate::execution::denial_cause::RelationalExecutionDenialCause::ProjectionContract,
        )?;
        Ok(projection_scope)
    }

    pub(super) fn admit_entity_projection_scope(
        &self,
        kind_id: KindId,
        projection_scope: &ProjectionAspectScope,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        admit_declared_projection_aspects(
            projection_scope,
            self.entity_aspect_plan(kind_id),
            ProjectionRecordDomain::Entity,
            kind_id,
        )
        .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::ProjectionContract)
    }

    pub(super) fn admit_relation_projection_scope(
        &self,
        kind_id: KindId,
        projection_scope: &ProjectionAspectScope,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        admit_declared_projection_aspects(
            projection_scope,
            self.relation_aspect_plan(kind_id),
            ProjectionRecordDomain::Relation,
            kind_id,
        )
        .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::ProjectionContract)
    }

    /// Exact number of executable aspect bindings installed for `kind_id`.
    pub fn entity_aspect_binding_count(&self, kind_id: KindId) -> usize {
        self.entity_aspect_plan(kind_id)
            .map_or(0, |plan| plan.executable_bindings.len())
    }

    pub(super) fn entity_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&crate::schema::data::LoweredAspectContractPlan> {
        match &self.basis {
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => basis
                .projection_root()
                .schema_authority()
                .entity_aspect_plan(kind_id),
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(_) => self
                .basis
                .root()
                .and_then(|root| root.schema_authority().entity_aspect_plan(kind_id)),
        }
    }

    pub(super) fn relation_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&crate::schema::data::LoweredAspectContractPlan> {
        match &self.basis {
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => basis
                .root()
                .schema_authority()
                .relation_aspect_plan(kind_id),
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(_) => self
                .basis
                .root()
                .and_then(|root| root.schema_authority().relation_aspect_plan(kind_id)),
        }
    }
}

#[path = "historical_denial.rs"]
pub mod historical_denial;

/// Operations used by the Relational owners.
pub trait VisibilityProjectionViewOwnerPort: Sized {
    fn is_from_runtime(&self, runtime_instance_id: u64) -> bool;
    fn selected_schema(&self) -> Option<SelectedProjectionSchema<'_>>;
}

impl VisibilityProjectionViewOwnerPort for VisibilityProjectionView {
    fn is_from_runtime(&self, runtime_instance_id: u64) -> bool {
        self.runtime_identity == runtime_instance_id
    }

    fn selected_schema(&self) -> Option<SelectedProjectionSchema<'_>> {
        let authority = self.basis.root()?.schema_authority();
        Some(SelectedProjectionSchema {
            registry: authority.registry(),
            plans: authority.aspect_plans(),
            version: authority.schema_version(),
            authority_digest: authority.authority_digest(),
        })
    }
}
