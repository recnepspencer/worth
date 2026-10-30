use worth_signal::facade::core::{
    ResourceSupersessionDecisionClass, ResourceSupersessionDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceSupersessionDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.supersession.new-generation-supersedes-prior"),
        class: ResourceSupersessionDecisionClass::NewGenerationSupersedesPrior,
        decision_digest: fake(),
    };
}
