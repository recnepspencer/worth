use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::declaration::{
    ComponentChildPolicy, ComponentDescriptor, ComponentHitTestContract, ComponentHitTestOrder,
    ComponentId, ComponentPropSchema, ComponentStateOwnership, MosaicRegionKindId,
};
use worth_ui_platform_pulse::product_world::{DashboardScrollPanel, PlatformPulseLogicalRect};

pub(super) fn register(
    mut builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    for (index, panel) in DashboardScrollPanel::ALL.into_iter().enumerate() {
        let id = format!("platform.pulse.component.{}", panel.owner());
        let [x, y, width, height] = panel.content_rect();
        let allocation =
            PlatformPulseLogicalRect::new(x.into(), y.into(), width.into(), height.into())
                .allocation();
        builder = builder.register_component(
            ComponentDescriptor::new(
                ComponentId::new(&id).unwrap(),
                ComponentPropSchema::named(format!("{id}.props")),
                ComponentChildPolicy::no_children(),
                ComponentStateOwnership::runtime_owned(),
            )
            .with_allocation_measurement_contract(allocation)
            .with_region_allocation(
                MosaicRegionKindId::new(panel.region()).unwrap(),
                panel.region_placement(),
            )
            .with_hit_test(ComponentHitTestContract::allocation_bounds(
                ComponentHitTestOrder::front_to_back(900 + index as u32),
                allocation,
            )),
        );
    }
    builder
}
