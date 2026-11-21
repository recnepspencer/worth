//! Directory admission retains the declared correction shape.
use super::*;
pub(super) fn admit<Schema: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
) -> Result<(), WorthQueryCorrectionDerivationDenial> {
    use WorthQueryCorrectionDerivationDenial as Denial;
    let runtime = &target.application().inner.runtime.product_runtime;
    if !runtime.is_directory_home() {
        return Ok(());
    }
    // The selected occurrence retains installed source meaning and genuine
    // program membership. H9-A's no-binding original-only shape is never enough.
    let association = source
        .occurrence()
        .disclosure()
        .map(|value| value.association());
    if let Some(binding) = association
        && (source.occurrence().original_binding() != Some(binding.source_type())
            || !source
                .occurrence()
                .original_program()
                .is_some_and(|program| {
                    program.declares_binding(binding.source_type())
                        && program.declares_binding(binding.inverse_type())
                }))
    {
        return Err(Denial::UnsupportedSourceMeaning);
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
    // CheckedCorrectionSource already carries exact owner-selected ancestry,
    // including inherited originals. Home preparation binds this target
    // observation; native branch-ID equality is not an ancestry condition.
    Ok(())
}
