pub(crate) use worth_relational::facade::runtime::RelationalRuntime;
pub(crate) use {
    worth_relational::facade::inspection::RelationalBranchSharingObservation,
    worth_relational::facade::inspection::RelationalStorageRegionLocator,
};

pub(crate) fn inspect_main_regions(
    runtime: &RelationalRuntime,
) -> Vec<RelationalStorageRegionLocator> {
    inspect_main_sharing(runtime).region_locators().to_vec()
}

pub(crate) fn inspect_main_sharing(
    runtime: &RelationalRuntime,
) -> RelationalBranchSharingObservation {
    runtime
        .observe_branch_sharing(&[runtime.main_branch_identity()])
        .expect("main branch sharing remains inspectable")
}
