use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor,
    ComponentHitTestContract, ComponentId, ComponentPropSchema, ComponentStateOwnership,
};

fn component() -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new("compile.visual.invalid_hit").unwrap(),
        ComponentPropSchema::named("compile.visual.invalid_hit.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn main() {
    let allocation = ComponentAllocationMeasurementContract::fill_viewport();
    let _ = component().with_hit_test(allocation);
    let _ = ComponentHitTestContract::allocation_bounds(0, allocation);
}
