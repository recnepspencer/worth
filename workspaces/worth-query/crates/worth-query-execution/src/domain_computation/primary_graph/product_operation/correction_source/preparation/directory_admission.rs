//! Landing 2a admits only correction shapes with qualified durable continuation.
use super::*;
pub(super) fn admit<Schema: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    ancestry: &ProductBranchAncestorSelection,
    lineage: &WorthQueryCorrectionLineageBinding,
    required_output: bool,
) -> Result<(), WorthQueryCorrectionDerivationDenial> {
    use crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial as Durable;
    use WorthQueryCorrectionDerivationDenial as Denial;
    let runtime = &target.application().runtime.product_runtime;
    if !runtime.is_directory_home() {
        return Ok(());
    }
    // The selected occurrence retains installed source meaning and genuine
    // program membership. H9-A's no-binding original-only shape is never enough.
    let association = source
        .occurrence()
        .disclosure()
        .map(|value| value.association());
    if let Some(binding) = association {
        if source.occurrence().original_binding() != Some(binding.source_type())
            || !source
                .occurrence()
                .original_program()
                .is_some_and(|program| {
                    program.declares_binding(binding.source_type())
                        && program.declares_binding(binding.inverse_type())
                })
        {
            return Err(Denial::UnsupportedSourceMeaning);
        }
    }
    runtime
        .admit_directory_correction_shape(
            source.occurrence().aftermath(),
            source
                .occurrence()
                .disclosure()
                .map(|value| value.association()),
        )
        .map_err(Denial::DurableStateAdmission)?;
    if required_output {
        runtime.admit_directory_capability(crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability::RequiredOutputDemand)
            .map_err(Denial::DurableStateAdmission)?;
    }
    if target.product().product_branch() != runtime.root_product_branch()
        || ancestry
            .source()
            .basis()
            .relational_basis()
            .identity()
            .branch_id()
            != target.product().relational_basis().identity().branch_id()
    {
        return Err(Denial::DurableStateAdmission(
            Durable::UnsupportedDivergentCorrection,
        ));
    }
    match lineage.role() {
        WorthQueryAftermathCausalRole::RecordedReverse => Ok(()),
        WorthQueryAftermathCausalRole::RecordedReapply => Err(Denial::DurableStateAdmission(
            Durable::UnsupportedCorrectionReapplication,
        )),
    }
}
