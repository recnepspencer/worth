//! First-use eligibility compares semantic source meaning on the exact target.
use super::disclosure::WorthQueryCorrectionDerivationDenial as Denial;
use super::*;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation;
use crate::domain_computation::primary_graph::{
    exact_basis_access::open_exact_basis_snapshot,
    output_reuse::{first_moved_fact, retained_fact_denial},
};
use worth_query_declaration::facade::application_aftermath::ApplicationInverseBinding;

pub(super) fn require_compatible_source<Schema, C>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<WorthQueryProgramSupportInterpretation, Denial>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
{
    let interpretation = require_target_program::<Schema, C>(target, source, required_root)?;
    let facts = source
        .occurrence()
        .commit_evidence()
        .rebased_source_facts()
        .map_err(|_| Denial::SemanticMaterialUnavailable)?;
    let application = target.application();
    let execution = phase
        .execution_request_for(&application.product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    application
        .primary_provider
        .graph
        .with_runtime_mut(&execution, |runtime| {
            let snapshot = open_exact_basis_snapshot(runtime, target.product().relational_basis())
                .map_err(Denial::Basis)?;
            let comparison = first_moved_fact(&execution, runtime, &snapshot, facts.iter());
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            match comparison {
                Ok(None) => Ok(()),
                Ok(Some(_)) => Err(Denial::SourceDiverged),
                Err(stop) => Err(Denial::DependencyComparison(retained_fact_denial(
                    stop,
                    "correction semantic dependency",
                ))),
            }
        })?;
    Ok(interpretation)
}

fn require_target_program<Schema, C>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<WorthQueryProgramSupportInterpretation, Denial>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
{
    let original = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .original_program()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let inspection = target
        .inspect_selected_program()
        .map_err(Denial::TargetProgram)?;
    let support = target
        .application()
        .installed_program_support()
        .ok_or(Denial::UnsupportedTargetProgram)?;
    let record = support
        .native_program(inspection.revision())
        .ok_or(Denial::UnsupportedTargetProgram)?;
    let declared = original
        .action_inventory()
        .resolve_unique_binding::<Schema, ApplicationInverseBinding<Schema, C>>()
        .map_err(Denial::InverseAction)?;
    let inverse_action = record
        .installed
        .action_inventory()
        .resolve_declared_action::<Schema, ApplicationInverseBinding<Schema, C>>(declared)
        .map_err(Denial::InverseAction)?;
    let requirements = support
        .adoption_requirements(
            target.application().installed_schema(),
            original.revision(),
            inspection.revision(),
        )
        .map_err(Denial::ProgramComparison)?;
    if !requirements.semantically_equivalent() {
        return Err(Denial::ProgramMeaningChanged(Box::new(requirements)));
    }
    let original_binding = crate::equality_type_id::EqualityTypeId::of::<C::Source>();
    let inverse =
        crate::equality_type_id::EqualityTypeId::of::<ApplicationInverseBinding<Schema, C>>();
    if required_root.is_none()
        && (record.requires_output_source(original_binding)
            || record.requires_output_source(inverse))
    {
        return Err(Denial::RequiredOutputReconciliation);
    }
    if let Some(root) = required_root {
        let delivery = inverse_action
            .delivery()
            .and_then(|delivery| delivery.output());
        if !delivery.is_some_and(|delivery| {
            !delivery.is_discovered() && delivery.root_type() == root.into_external()
        }) {
            return Err(Denial::InverseAction(worth_query_installation::facade::WorthQueryActionResolutionDenial::DeliveryMismatch));
        }
        if !record.root_graph_types.contains(&root) || !record.requires_output_source(inverse) {
            return Err(Denial::UnsupportedTargetProgram);
        }
    }
    if !record.declares_action(inverse) {
        return Err(Denial::UnsupportedTargetProgram);
    }
    support
        .retain_interpretation(inspection.revision())
        .ok_or(Denial::UnsupportedTargetProgram)
}
