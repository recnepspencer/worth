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
    outputs: Option<&super::WorthQueryHistoryOutputProfile>,
) -> Result<(), Denial> {
    let definition = installed.definition().parts();
    let navigation = definition
        .navigation
        .as_ref()
        .ok_or_else(|| correction(Eligibility::NotDeclared))?;
    if !matches!(
        definition.model_domain,
        ApplicationHistoryModelDomain::ScopeEntityRecord
            | ApplicationHistoryModelDomain::OwnedModelGraph
    ) || (definition.model_domain == ApplicationHistoryModelDomain::ScopeEntityRecord
        && !definition.ownership.is_empty())
    {
        return Err(correction(Eligibility::ModelDomain));
    }
    if source.model != *model {
        return Err(Denial::Document(
            super::super::WorthQueryDocumentHistoryDenial::ForeignModel,
        ));
    }
    if source.root_posture.is_some() && outputs.is_none() {
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
        .runtime
        .primary_provider
        .graph
        .layout
        .is_live_control_entity_kind(model.kind())
    {
        return Err(correction(Eligibility::LiveControlDomain));
    }
    application
        .runtime
        .product_runtime
        .require_live_product_occurrence(
            source.source.product_observation().lifecycle_incarnation(),
        )
        .map_err(Denial::ProductAdmission)?;
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
    let expected_output = if let Some(profile) = outputs {
        if profile.model != *model || profile.program != *current_program.revision() {
            return Err(correction(Eligibility::ProgramChanged));
        }
        if let super::output_profile::HistoryOutputRegistration::ActionDelivery(delivery_index) =
            profile.registration
        {
            Some(
                supported
                    .installed
                    .action_inventory()
                    .declarations()
                    .get(delivery_index)
                    .and_then(|action| action.delivery())
                    .and_then(|delivery| delivery.output())
                    .filter(|output| {
                        output.root_type() == profile.root
                            && output.read().is_none_or(|read| read == &access)
                    })
                    .ok_or_else(|| correction(Eligibility::RequiredOutputs))?,
            )
        } else {
            None
        }
    } else {
        None
    };
    if let Some(profile) = outputs.filter(|profile| profile.is_output_forest()) {
        let execution = phase
            .checked_execution_request_for(&application.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let work = profile
            .tree_profiles()
            .len()
            .checked_mul(
                supported
                    .installed
                    .action_inventory()
                    .declarations()
                    .len()
                    .saturating_add(1),
            )
            .and_then(|work| u64::try_from(work).ok())
            .ok_or(Denial::OutputLineage(
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
            ))?;
        execution.run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        }).map_err(|cause| Denial::Stopped(crate::domain_computation::primary_graph::application_contribution::advancement_denial(cause)))?
            .0.map_err(|cause| Denial::Stopped(crate::domain_computation::primary_graph::application_contribution::advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))))?;
    }
    for action in supported.installed.action_inventory().declarations() {
        let output = action.delivery().and_then(|delivery| delivery.output());
        let relevant = action.operation_identifier() == navigation.parts().operation
            || action.mutation_binding_type().is_some_and(|binding| {
                installed
                    .operation_descriptors()
                    .iter()
                    .any(|operation| operation.binding_type() == binding)
            });
        if relevant
            && (action.required_output_source() || output.is_some())
            && if let Some(profile) = outputs.filter(|profile| profile.uses_owned_model_graph()) {
                output.is_some_and(|output| {
                    output.read().is_some_and(|read| read != &access)
                        || !profile
                            .tree_profiles()
                            .iter()
                            .any(|tree| tree.root == output.root_type())
                })
            } else {
                expected_output.is_none() || output != expected_output
            }
        {
            return Err(correction(Eligibility::RequiredOutputs));
        }
        if relevant && action.workflow_execution_requirements().is_applicable() {
            return Err(correction(Eligibility::Workflow));
        }
    }
    let execution = phase
        .checked_execution_request_for(&application.runtime.product_runtime)
        .map_err(Denial::Stopped)?;
    // This real admission on C catches medium/schema/source eligibility before
    // fork. Actual preparation later repeats admission on the new C-derived target.
    application
        .runtime
        .primary_provider
        .graph
        .with_runtime(&execution, |native| {
            if definition.model_domain == ApplicationHistoryModelDomain::OwnedModelGraph {
                let layout = &application.runtime.primary_provider.graph.layout;
                let ownership =
                    layout
                        .history_ownership(&definition.identity)
                        .ok_or(Denial::Document(
                            super::super::WorthQueryDocumentHistoryDenial::InstalledBindingMismatch,
                        ))?;
                let graph = native
                    .prepare_owned_model_graph_selection(
                        &execution,
                        current.product().relational_basis(),
                        historical.product().relational_basis(),
                        ownership,
                        layout.installed_history_ownership(),
                        model.entity(),
                    )
                    .map_err(Denial::OwnedGraph)?;
                let transaction = native
                    .begin_branch_transaction(
                        current.product().relational_basis(),
                        RelationalTransactionIntent::ordinary(),
                    )
                    .map_err(Denial::Transaction)?;
                let prepared = super::output_custody::prepare(
                    &execution,
                    native,
                    application
                        .runtime
                        .primary_provider
                        .graph
                        .application_publication
                        .as_deref(),
                    transaction,
                    graph,
                )?;
                if outputs.is_none()
                    && prepared
                        .1
                        .as_ref()
                        .is_some_and(|custody| !custody.routes.is_empty())
                {
                    return Err(correction(Eligibility::RequiredOutputs));
                }
                drop(prepared);
                return Ok(());
            }
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
