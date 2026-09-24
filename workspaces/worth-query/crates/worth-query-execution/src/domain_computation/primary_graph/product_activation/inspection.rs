use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// The rostered program actually carried by one selected product occurrence.
/// This is descriptive operator evidence; it grants no mutation or adoption
/// authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySelectedProgramInspection {
    revision: ApplicationProgramRevision,
}

impl WorthQuerySelectedProgramInspection {
    pub const fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySelectedProgramInspectionDenial {
    ProgramSupportUnavailable,
    ProgramActivationUnavailable,
    ProgramActivationUnreadable,
    ProgramActivationUnrostered,
}

pub(in crate::domain_computation::primary_graph) fn inspect_selected_program<
    Schema: ApplicationSchema,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    version: worth_relational::facade::identity::VersionId,
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
    let rendering = graph
        .with_runtime(|runtime| {
            let record = runtime
                .read_truth()
                .visible_entity_at_version(activation, version)?;
            let state = record.authoritative_aspect_state.as_ref()?;
            let value = state.get(layout.program_revision_locator.aspect().aspect_key())?;
            let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
                return None;
            };
            let field = layout
                .program_revision_locator
                .field_path()
                .fields()
                .first()?;
            fields.get(field).cloned()
        })
        .ok_or(WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnreadable)?;
    let revision = support
        .rostered_for_rendering(&rendering)
        .ok_or(WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnrostered)?
        .revision()
        .clone();
    Ok(WorthQuerySelectedProgramInspection { revision })
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn retain_selected_program_interpretation(
        &self,
        version: worth_relational::facade::identity::VersionId,
    ) -> Result<
        WorthQueryRetainedSelectedProgramInspection,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        let inspection = inspect_selected_program(self, version);
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
        version: worth_relational::facade::identity::VersionId,
        basis: &mut crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    ) -> Result<
        Result<WorthQuerySelectedProgramInspection, WorthQuerySelectedProgramInspectionDenial>,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        let (selected, interpretation) = self
            .retain_selected_program_interpretation(version)?
            .into_parts();
        if let Some(interpretation) = interpretation {
            basis.bind_program_interpretation(interpretation);
        }
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
