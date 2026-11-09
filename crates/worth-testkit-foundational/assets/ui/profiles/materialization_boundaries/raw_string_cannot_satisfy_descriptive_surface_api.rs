use worth_foundational::{
    plan_selected_foundational_profile_materialization, FoundationalDescriptiveSurface,
    MaterializedFoundationalProfileSet, SupportArtifactTarget,
};

fn main() {
    let profile: MaterializedFoundationalProfileSet = supplied();
    let raw = ["history"];
    let _ =
        plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(&profile, &raw);
    let _typed: FoundationalDescriptiveSurface = raw[0];
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
