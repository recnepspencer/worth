use worth_query_declaration::facade::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::storage::RecordLifecycleState;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// The rostered program actually carried by one selected product occurrence.
/// This is descriptive operator evidence; it grants no mutation or adoption
/// authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySelectedProgramInspection {
    identity: ApplicationProgramIdentity,
    revision: ApplicationProgramRevision,
}

impl WorthQuerySelectedProgramInspection {
    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }

    pub const fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }
}

/// Why the program carried by a selected product could not be inspected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySelectedProgramInspectionDenial {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
    /// The runtime has no installed program support.
    ProgramSupportUnavailable,
    /// No program activation is published.
    ProgramActivationUnavailable,
    /// The product's program activation record could not be read.
    ProgramActivationUnreadable,
    /// The product's program revision is not on the installed roster.
    ProgramActivationUnrostered,
}

pub(in crate::domain_computation::primary_graph) fn inspect_selected_program<
    Schema: ApplicationSchema,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<WorthQuerySelectedProgramInspection, WorthQuerySelectedProgramInspectionDenial> {
    let support = application
        .installed_program_support()
        .ok_or(WorthQuerySelectedProgramInspectionDenial::ProgramSupportUnavailable)?;
    let activation = support
        .activation()
        .published()
        .ok_or(WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnavailable)?;
    let graph = &application.primary_provider.graph;
    let layout = graph.layout.program_activation().clone();
    let locator = &layout.program_revision_locator;
    // The activation record is read on the selected branch's own root: a
    // version alone would also see a sibling's later adoption.
    let rendering =
        graph
            .with_runtime(|runtime| {
                let field = locator.field_path().fields().first()?.clone();
                let aspect = locator.aspect().aspect_key();
                let scope = ProjectionAspectScope::from_requirements([
                    ProjectionAspectRequirement::fields(aspect.clone(), [field.clone()]),
                ]);
                runtime
                    .read_truth()
                    .project_observation(&basis.observation())
                    .ok()?
                    .entity_record_with_projection_scope(activation, scope, |record| {
                        if record.kind_id() != layout.entity_kind
                            || record.lifecycle() != RecordLifecycleState::Live
                        {
                            return None;
                        }
                        record.aspect_field_value(aspect, &field).cloned()
                    })
            })?
            .ok_or(WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnreadable)?;
    let entry = support
        .rostered_for_rendering(&rendering)
        .ok_or(WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnrostered)?;
    Ok(WorthQuerySelectedProgramInspection {
        identity: entry.identity().clone(),
        revision: *entry.revision(),
    })
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn retain_selected_program_interpretation(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<
        WorthQueryRetainedSelectedProgramInspection,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        let inspection = inspect_selected_program(self, basis);
        let interpretation = if let (Some(support), Ok(selected)) =
            (self.installed_program_support(), &inspection)
        {
            Some(
                support.retain_interpretation(selected.revision()).ok_or(
                    crate::basis::WorthQueryProductBranchAdmissionDenial::ObservationRejected,
                )?,
            )
        } else {
            None
        };
        Ok(WorthQueryRetainedSelectedProgramInspection {
            inspection,
            interpretation,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn bind_selected_program_interpretation(
        &self,
        relational: &AdmittedRelationalBranchBasis,
        basis: &mut crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    ) -> Result<
        Result<WorthQuerySelectedProgramInspection, WorthQuerySelectedProgramInspectionDenial>,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        if basis.identity().runtime_instance_id() != relational.identity().runtime_instance_id()
            || basis.identity().branch_id() != relational.identity().branch_id()
            || basis.identity().descriptor() != relational.descriptor()
            || basis.version_id() != relational.observation().version_id()
        {
            return Err(crate::basis::WorthQueryProductBranchAdmissionDenial::ObservationRejected);
        }
        let (selected, interpretation) = self
            .retain_selected_program_interpretation(relational)?
            .into_parts();
        if let Some(interpretation) = interpretation {
            basis.bind_program_interpretation(interpretation);
        }
        let marked = basis.mark_selected_program_inspected(relational);
        debug_assert!(marked, "the checked selected program belongs to this basis");
        Ok(selected)
    }
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryRetainedSelectedProgramInspection
{
    inspection:
        Result<WorthQuerySelectedProgramInspection, WorthQuerySelectedProgramInspectionDenial>,
    interpretation:
        Option<super::super::program_occurrence::WorthQueryProgramSupportInterpretation>,
}

impl WorthQueryRetainedSelectedProgramInspection {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        Result<WorthQuerySelectedProgramInspection, WorthQuerySelectedProgramInspectionDenial>,
        Option<super::super::program_occurrence::WorthQueryProgramSupportInterpretation>,
    ) {
        (self.inspection, self.interpretation)
    }
}

impl From<crate::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQuerySelectedProgramInspectionDenial
{
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::Handle(denial)
    }
}
