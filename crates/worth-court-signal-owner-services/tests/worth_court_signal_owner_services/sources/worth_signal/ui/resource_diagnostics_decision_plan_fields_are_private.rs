use worth_signal::facade::core::{
    ResourceDiagnosticsDecisionClass, ResourceDiagnosticsDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceDiagnosticsDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.diagnostics.budgeted-expansion"),
        class: ResourceDiagnosticsDecisionClass::BudgetedExpansion,
        max_replay_reconstruction_width: Some(5),
        decision_digest: fake(),
    };
}
