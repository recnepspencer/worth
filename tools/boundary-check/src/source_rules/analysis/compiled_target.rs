//! The existing module walk rooted at one explicitly selected Cargo target.
use super::crate_modules::{parse_module_tree, GovernedCrate, ModuleFilter, ModuleGraph};
use std::{collections::BTreeMap, path::Path};
pub(crate) fn parse_compiled_target_where(
    owner: &GovernedCrate,
    target: &Path,
    descend: ModuleFilter<'_>,
) -> Result<ModuleGraph, String> {
    let mut modules = BTreeMap::new();
    parse_module_tree(
        &owner.crate_root,
        target,
        target.parent().unwrap(),
        Vec::new(),
        true,
        &mut modules,
        descend,
    )?;
    Ok(ModuleGraph { modules })
}
