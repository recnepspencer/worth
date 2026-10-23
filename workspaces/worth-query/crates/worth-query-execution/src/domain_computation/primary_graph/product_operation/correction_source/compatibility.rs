//! First-use eligibility compares semantic source meaning on the exact target.
use super::disclosure::WorthQueryCorrectionDerivationDenial as Denial;
use super::*;
use crate::domain_computation::primary_graph::{
    exact_basis_access::open_exact_basis_snapshot,
    output_reuse::{first_moved_fact, retained_fact_denial},
};
use worth_query_declaration::facade::application_aftermath::ApplicationInverseBinding;

pub(super) fn require_compatible_source<Schema, C>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
) -> Result<(), Denial>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
{
    require_target_program::<Schema, C>(target, source)?;
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
        })
}

fn require_target_program<Schema, C>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
) -> Result<(), Denial>
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
    let record = target
        .application()
        .installed_program_support()
        .and_then(|support| support.native_program(inspection.revision()))
        .ok_or(Denial::UnsupportedTargetProgram)?;
    if !original.is_same_installed_program(record) {
        return Err(Denial::UnsupportedTargetProgram);
    }
    let original_binding = crate::equality_type_id::EqualityTypeId::of::<C::Source>();
    if record.requires_output_source(original_binding) {
        return Err(Denial::RequiredOutputReconciliation);
    }
    let inverse =
        crate::equality_type_id::EqualityTypeId::of::<ApplicationInverseBinding<Schema, C>>();
    if record.requires_output_source(inverse) {
        return Err(Denial::RequiredOutputReconciliation);
    }
    if !record.declares_action(inverse) {
        return Err(Denial::UnsupportedTargetProgram);
    }
    Ok(())
}
