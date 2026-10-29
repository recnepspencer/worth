use worth_foundational::{
    profiles,
    profiles_api::{common_path, lower_lane::materialization as lower_materialization},
    FoundationalDescriptiveSurface, FoundationalObservationDisposition,
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
    > = lower_materialization::plan_selected_foundational_profile_materialization_with_disposition;

    let _common = common_path::profiles().materialization();
    let _grouped = profiles().materialization();
}
