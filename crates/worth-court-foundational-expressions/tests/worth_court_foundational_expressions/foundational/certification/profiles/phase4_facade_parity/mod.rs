use {
    worth_foundational::profiles, worth_foundational::profiles_api::common_path,
    worth_foundational::profiles_api::lower_lane::materialization as lower_materialization,
    worth_foundational::FoundationalDescriptiveSurface,
    worth_foundational::FoundationalObservationDisposition,
};

#[test]
fn lower_lane_materialization_matches_root_and_common_paths() {
    let _root: fn(
        &worth_foundational::MaterializedFoundationalProfileSet,
        &[FoundationalDescriptiveSurface],
        FoundationalObservationDisposition,
    ) -> Result<
        worth_foundational::FoundationalProfileMaterializationPlan<
            worth_foundational::ProofBearingArtifactTarget,
        >,
        worth_foundational::FoundationalMaterializationPlanningDenial,
    > = worth_foundational::plan_selected_foundational_profile_materialization_with_disposition;
    let _lower: fn(
        &worth_foundational::MaterializedFoundationalProfileSet,
        &[FoundationalDescriptiveSurface],
        FoundationalObservationDisposition,
    ) -> Result<
        worth_foundational::FoundationalProfileMaterializationPlan<
            worth_foundational::ProofBearingArtifactTarget,
        >,
        worth_foundational::FoundationalMaterializationPlanningDenial,
    > = worth_foundational::profiles_api::lower_lane::materialization::plan_selected_foundational_profile_materialization_with_disposition;

    let _common = worth_foundational::profiles_api::common_path::profiles().materialization();
    let _grouped = profiles().materialization();
}
