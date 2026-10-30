use worth_signal::facade::{
    CommittedResourceCompletionArtifact, ResourceLifecycleTransition, StagedResourceCompletionEffect,
};

fn forged_staged_effect() -> StagedResourceCompletionEffect {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn main() {
    let _ = CommittedResourceCompletionArtifact {
        staged_effect: forged_staged_effect(),
        lifecycle_transition: forged_transition(),
    };
}
