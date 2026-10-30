use worth_foundational::{
    plan_selected_foundational_profile_materialization, FoundationalDescriptiveSurface,
    MaterializedFoundationalProfileSet, SupportArtifactTarget,
};

fn main() {
    let profile: MaterializedFoundationalProfileSet = panic!("type-check only");
    let raw = ["history"];
    let _ = plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(
        &profile,
        &raw,
    );
    let _typed: FoundationalDescriptiveSurface = raw[0];
}
