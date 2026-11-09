//! An original-request read lens over A, with no live mutation or provider ports.
use super::super::super::super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::*;
use worth_relational::facade::{
    identity::KindId,
    runtime::{
        BoundedEntityKindTruthRead, BoundedRelationKindTruthRead, RelationalPinnedRead,
        VisibilityProjectionView,
    },
};

pub struct WorthQueryHomeProgramTransitionReadSource<'owner, 'scope> {
    pub(super) view: VisibilityProjectionView,
    metadata: RelationalPinnedRead<()>,
    pub(super) layout: &'owner WorthQueryPrimaryGraphLayout,
    phase: &'owner WorthQueryBootstrapAdvancementPhase<'scope>,
}
impl WorthQueryHomeProgramTransitionReadSource<'_, '_> {
    pub fn schema_registry(&self) -> &worth_relational::facade::schema::RelationalSchemaRegistry {
        self.metadata.schema()
    }
    pub fn entity_kind(&self, name: &str) -> Option<KindId> {
        self.layout.entity_kind(name)
    }
    pub fn planned_field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<worth_foundational::facade::AspectFieldLocator> {
        self.layout
            .field_locator(entity, aspect, field)
            .map(planned)
    }
    pub fn entities(
        &self,
        name: &str,
        maximum_work_units: usize,
    ) -> Result<BoundedEntityKindTruthRead, WorthQueryPrimaryGraphInstallationDenial> {
        let kind = self
            .layout
            .entity_kind(name)
            .ok_or_else(|| denied("transition reads only declared A entity kinds"))?;
        self.view
            .bounded_entities_of_kind(&self.phase.execution_request(), kind, maximum_work_units)
            .map_err(super::super::super::super::bootstrap::request_read_denial::read)?
            .map_err(|_| denied("transition entity read exceeded its declared bound"))
    }
    pub fn relations(
        &self,
        name: &str,
        maximum_work_units: usize,
    ) -> Result<BoundedRelationKindTruthRead, WorthQueryPrimaryGraphInstallationDenial> {
        let kind = self
            .layout
            .relation(name)
            .ok_or_else(|| denied("transition reads only declared A relation kinds"))?
            .kind;
        self.view
            .bounded_relations_of_kind(&self.phase.execution_request(), kind, maximum_work_units)
            .map_err(super::super::super::super::bootstrap::request_read_denial::read)?
            .map_err(|_| denied("transition relation read exceeded its declared bound"))
    }
    pub(super) fn request(&self) -> worth_execution::ExecutionRequest<'_, '_> {
        self.phase.execution_request()
    }
}

pub struct WorthQueryHomeProgramTransitionTarget<'target> {
    pub(super) layout: &'target WorthQueryPrimaryGraphLayout,
    registry: &'target worth_relational::facade::schema::RelationalSchemaRegistry,
}
impl WorthQueryHomeProgramTransitionTarget<'_> {
    pub fn schema_registry(&self) -> &worth_relational::facade::schema::RelationalSchemaRegistry {
        self.registry
    }
    pub fn entity_kind(&self, name: &str) -> Option<KindId> {
        self.layout.entity_kind(name)
    }
    pub fn planned_field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<worth_foundational::facade::AspectFieldLocator> {
        self.layout
            .field_locator(entity, aspect, field)
            .map(planned)
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedExistingHomeSignalProgram<Schema, Program>
{
    pub fn home_program_transition_source<'owner, 'scope>(
        &'owner self,
        phase: &'owner WorthQueryBootstrapAdvancementPhase<'scope>,
        native: &RelationalRuntime,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<
        WorthQueryHomeProgramTransitionReadSource<'owner, 'scope>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        let (_, receipt) = self.predecessor_native_meaning();
        if receipt.runtime_instance_id() != native.runtime_instance_id() {
            return Err(denied(
                "transition source belongs to another actual Native receiver",
            ));
        }
        let view = native
            .read_truth()
            .project_observation(&basis.observation())
            .map_err(|cause| denied(format!("transition original source refused: {cause:?}")))?;
        let metadata = native
            .read_truth()
            .pin_view_read_capability(&view, &[], &phase.execution_request())
            .map_err(|cause| denied(format!("transition original metadata refused: {cause:?}")))?;
        Ok(WorthQueryHomeProgramTransitionReadSource {
            view,
            metadata,
            layout: self.predecessor_native_meaning().0,
            phase,
        })
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
    pub fn authoring_target(&self) -> WorthQueryHomeProgramTransitionTarget<'_> {
        WorthQueryHomeProgramTransitionTarget {
            layout: &self._pending._layout,
            registry: &self
                .contracts
                .receiver_configuration()
                .expect("Query prepared the complete receiving configuration")
                .schema
                .registry,
        }
    }
}
fn planned(
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> worth_foundational::facade::AspectFieldLocator {
    worth_relational::facade::transactions::planned_aspect_field_locator(
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    )
}
pub(super) fn denied(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(super::super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember, detail)
}
