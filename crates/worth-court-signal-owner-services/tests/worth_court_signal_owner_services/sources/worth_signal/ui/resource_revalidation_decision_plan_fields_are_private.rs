use worth_signal::facade::core::{
    ResourceRevalidationDecisionClass, ResourceRevalidationDecisionPlan,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _plan = ResourceRevalidationDecisionPlan {
        descriptor_id: fake(),
        semantic_name: String::from("signal.resource.revalidation.explicit-or-active-handle-forced"),
        class: ResourceRevalidationDecisionClass::ExplicitOrActiveHandleForced,
        permits_active_handle_forcing: true,
        permits_observer_demand_revalidation: false,
        decision_digest: fake(),
    };
}
