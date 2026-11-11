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
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) actor: worth_proof::AdmittedWorkflowEndingActor<
        crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor,
        worth_relational::facade::runtime::RelationalHomeProgramPlanningSource,
    >,
}

#[derive(Debug)]
pub enum QueryHomeProgramTransitionSourceDenial {
    Native(worth_relational::facade::runtime::RelationalHomeProgramContractDenial),
    Meaning(WorthQueryPrimaryGraphInstallationDenial),
    NonDurableWorldOwner,
    ForeignWorldOwner,
    Activation(super::super::super::super::bootstrap::branch_activation_history::QueryHomeProgramActivationDenial),
    Correction(crate::domain_computation::primary_graph::provider::aftermath_causality::readmission::QueryHomeCorrectionLineageDenial),
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
    ReadmittedExistingHomeNativeMeaning<Schema, Program>
{
    pub fn home_program_transition_source<'owner, 'scope>(
        &'owner self,
        phase: &'owner WorthQueryBootstrapAdvancementPhase<'scope>,
        native: &'owner RelationalRuntime,
        source: &'owner worth_runtime_world::facade::ProductBranchObservation,
        admitted: worth_proof::AdmittedHomeProgramTransitionPlanningRequest<
            worth_relational::facade::runtime::RelationalHomeProgramPlanningSource,
        >,
        observation: &worth_runtime_world::facade::RuntimeWorldObservationPort,
        inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
    ) -> Result<
        WorthQueryHomeProgramTransitionReadSource<'owner, 'scope>,
        QueryHomeProgramTransitionSourceDenial,
    > {
        let (layout, receipt) = self.predecessor_native_meaning();
        admit_source_lens(
            phase,
            native,
            source,
            admitted,
            layout,
            receipt,
            self.predecessor_program_support().1,
            observation,
            inspection,
        )
    }
}

fn admit_source_lens<'owner, 'scope, Schema>(
    phase: &'owner WorthQueryBootstrapAdvancementPhase<'scope>,
    native: &'owner RelationalRuntime,
    source: &'owner worth_runtime_world::facade::ProductBranchObservation,
    admitted: worth_proof::AdmittedHomeProgramTransitionPlanningRequest<
        worth_relational::facade::runtime::RelationalHomeProgramPlanningSource,
    >,
    layout: &'owner WorthQueryPrimaryGraphLayout,
    receipt: &worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt,
    roster: &worth_query_installation::facade::WorthQueryProgramSupportRoster<Schema>,
    observation: &worth_runtime_world::facade::RuntimeWorldObservationPort,
    inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
) -> Result<
    WorthQueryHomeProgramTransitionReadSource<'owner, 'scope>,
    QueryHomeProgramTransitionSourceDenial,
> {
    use QueryHomeProgramTransitionSourceDenial as Denial;
    let basis = source.basis().relational_basis();
    // Incarnation/owner admission precedes every scope test and truth read.
    native
        .admit_home_program_transition_planning_request(&admitted, basis)
        .map_err(Denial::Native)?;
    let owner = source
        .owner_identity()
        .durable()
        .ok_or(Denial::NonDurableWorldOwner)?;
    if owner != admitted.source().actor() {
        return Err(Denial::ForeignWorldOwner);
    }
    if receipt.runtime_instance_id() != native.runtime_instance_id() {
        return Err(Denial::Meaning(denied(
            "transition source belongs to another actual Native receiver",
        )));
    }
    super::super::super::super::bootstrap::branch_activation_history::verify_predecessor(
        native,
        layout,
        &phase.execution_request(),
        source,
        roster,
        observation,
        inspection,
    )
    .map_err(Denial::Activation)?;
    crate::domain_computation::primary_graph::provider::aftermath_causality::readmission::verify_source(
        &phase.execution_request(), native, layout, inspection, observation,
    ).map_err(Denial::Correction)?;
    let view = native
        .read_truth()
        .project_observation(&basis.observation())
        .map_err(|cause| {
            Denial::Meaning(denied(format!(
                "transition original source refused: {cause:?}"
            )))
        })?;
    let metadata = native
        .read_truth()
        .pin_view_read_capability(&view, &[], &phase.execution_request())
        .map_err(|cause| {
            Denial::Meaning(denied(format!(
                "transition original metadata refused: {cause:?}"
            )))
        })?;
    Ok(WorthQueryHomeProgramTransitionReadSource {
        view,
        metadata,
        layout,
        phase,
        branch: crate::basis::WorthQueryProductBranch::from_occurrence(
            source.lifecycle_incarnation(),
        ),
        actor: worth_proof::AdmittedWorkflowEndingActor::from_home_program_transition(admitted),
    })
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
