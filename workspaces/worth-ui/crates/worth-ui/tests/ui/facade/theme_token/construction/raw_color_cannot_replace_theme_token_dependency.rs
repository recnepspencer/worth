use worth_ui::facade::declaration::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentSemanticTextContract, ComponentStateOwnership,
};

fn main() {
    let _descriptor = ComponentDescriptor::new(
        ComponentId::new("component.label").unwrap(),
        ComponentPropSchema::named("component.label.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_semantic_text(ComponentSemanticTextContract::body_default("#ffffff", 1));
}
