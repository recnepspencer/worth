//! Reverse coverage of complete installed operation/workflow target locations.
use super::*;
use std::collections::BTreeSet;
use worth_query_installation::facade::WorthQueryInstalledPackageIndex;

pub(super) fn validate(
    declared: &BTreeMap<String, DeclaredConditionalBinding>,
    index: &WorthQueryInstalledPackageIndex,
    owner: &str,
    schema: &str,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let installed = index
        .installed_conditional_targets_for_schema(owner, schema)
        .into_iter()
        .map(|(binding, location)| (binding.clone(), location))
        .collect::<BTreeSet<_>>();
    // Distinct exact binding scopes may share one target. This compares the
    // full portable binding and complete location, never a name/count proxy.
    let selected = declared
        .values()
        .map(|binding| {
            (
                binding.contract.operation_binding(),
                binding.contract.target().clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    if installed == selected {
        return Ok(());
    }
    let mismatch = installed
        .symmetric_difference(&selected)
        .next()
        .expect("different target sets have a difference");
    Err(denial(
        DenialKind::MissingConditionalBinding,
        format!(
            "incomplete conditional target coverage: {}/{}/{:?}",
            mismatch.0.application_operation(),
            mismatch.0.domain_operation_slot(),
            mismatch.1
        ),
    ))
}
