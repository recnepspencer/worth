//! Qualify actual installed meaning and both real sources before creating a branch.
use super::super::{HistoryModelIdentity, WorthQueryHistoryPreview};
use super::{
    WorthQueryHistoryNavigationDenial as Denial,
    WorthQueryHistoryNavigationEligibilityDenial as Eligibility,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryAccessBinding, ApplicationHistoryBinding, ApplicationHistoryModelDomain,
    },
    application_query::ApplicationQueryBinding,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledDocumentHistory};
use worth_relational::facade::mvcc::{
    RelationalRetainedEntitySelectionPreparationError as Preparation, RelationalTransactionIntent,
};

fn correction(cause: Eligibility) -> Denial {
    Denial::RequiresSemanticCorrection(cause)
}

pub(super) fn require<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(
    phase: &WorthQueryAdvancementPhase<'_>,
    current: &WorthQuerySelectedProductOperation<'_, S>,
    installed: &WorthQueryInstalledDocumentHistory<S, H>,
    model: &HistoryModelIdentity,
    source: &WorthQueryHistoryPreview,
) -> Result<(), Denial> {
    let definition = installed.definition().parts();
    let navigation = definition
        .navigation
        .as_ref()
        .ok_or_else(|| correction(Eligibility::NotDeclared))?;
    if definition.model_domain != ApplicationHistoryModelDomain::ScopeEntityRecord {
        return Err(correction(Eligibility::ModelDomain));
    }
    if source.model != *model {
        return Err(Denial::Document(
            super::super::WorthQueryDocumentHistoryDenial::ForeignModel,
        ));
    }
    if source.root_posture.is_some() {
        return Err(correction(Eligibility::RequiredOutputs));
    }
    if installed.has_external_operation() {
        return Err(correction(Eligibility::ExternalEffects));
    }
    if installed.operation_descriptors().iter().any(|operation| {
        operation.workflow_control()
            || operation.requires_workflow_authority()
            || operation.workflow_execution_requirements().is_applicable()
    }) {
        return Err(correction(Eligibility::Workflow));
    }
    let application = current.application();
    if application
        .primary_provider
        .graph
        .layout
        .is_live_control_entity_kind(model.kind())
    {
        return Err(correction(Eligibility::LiveControlDomain));
    }
    let historical = application
        .select_application_read_observation(phase, &source.source)
        .map_err(Denial::ProductAdmission)?;
    if historical
        .product()
        .observation()
        .basis()
        .signal_basis()
        .descriptor()
        != current
            .product()
            .observation()
            .basis()
            .signal_basis()
            .descriptor()
    {
        return Err(correction(Eligibility::SignalChanged));
    }
    let current_program = current
        .inspect_selected_program()
        .map_err(|_| correction(Eligibility::ProgramUnavailable))?;
    let historical_program = historical
        .inspect_selected_program()
        .map_err(|_| correction(Eligibility::ProgramUnavailable))?;
    if current_program != historical_program {
        return Err(correction(Eligibility::ProgramChanged));
    }
    let supported = application
        .installed_program_support()
        .and_then(|support| support.native_program(current_program.revision()))
        .ok_or_else(|| correction(Eligibility::ProgramUnavailable))?;
    let access = ApplicationHistoryAccessBinding::<S, H>::descriptor();
    for action in supported.installed.action_inventory().declarations() {
        let output = action.delivery().and_then(|delivery| delivery.output());
        let relevant = action.operation_identifier() == navigation.parts().operation
            || action.mutation_binding_type().is_some_and(|binding| {
                installed
                    .operation_descriptors()
                    .iter()
                    .any(|operation| operation.binding_type() == binding)
            })
            || output
                .and_then(|output| output.read())
                .is_some_and(|read| *read == access);
        if relevant && (action.required_output_source() || output.is_some()) {
            return Err(correction(Eligibility::RequiredOutputs));
        }
        if relevant && action.workflow_execution_requirements().is_applicable() {
            return Err(correction(Eligibility::Workflow));
        }
    }
    let execution = phase
        .checked_execution_request_for(&application.product_runtime)
        .map_err(Denial::Stopped)?;
    // This real admission on C catches medium/schema/source eligibility before
    // fork. Actual preparation later repeats admission on the new C-derived target.
    application
        .primary_provider
        .graph
        .with_runtime(&execution, |native| {
            let transaction = native
                .begin_branch_transaction(
                    current.product().relational_basis(),
                    RelationalTransactionIntent::ordinary(),
                )
                .map_err(Denial::Transaction)?;
            let selection = native
                .preparation_port()
                .admit_retained_entity_selection(
                    transaction,
                    historical.product().relational_basis(),
                    model.entity(),
                )
                .map_err(|cause| Denial::Preparation(Preparation::Admission(cause)))?;
            drop(selection);
            Ok(())
        })
}
