mod current_selector;
mod manifest;
mod previous_selector;
mod routing_block;
mod staged_current_selector;

pub(crate) use current_selector::{
    admit_current_root_selector, IntegrityAdmittedCurrentRootSelector,
};
pub(crate) use manifest::{admit_root_manifest, IntegrityAdmittedRootManifest};
pub(crate) use previous_selector::{
    admit_previous_root_selector, IntegrityAdmittedPreviousRootSelector,
};
pub(crate) use routing_block::IntegrityAdmittedRootRoutingBlock;
pub(crate) use staged_current_selector::IntegrityAdmittedStagedCurrentSelector;

#[cfg(test)]
pub(super) fn owner_valid_compile_contracts() {
    current_selector::owner_valid_compile_contract();
    previous_selector::owner_valid_compile_contract();
    manifest::owner_valid_compile_contract();
    routing_block::owner_valid_compile_contract();
}
