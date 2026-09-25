use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentDescriptor, ComponentHitTestContract,
    ComponentHitTestOrder, MosaicRegionKindId,
};
use worth_ui_platform_pulse::product_world::DashboardScrollPanel;

/// Makes a list's content container the owner that scrolls it: it places the
/// list's region and hit-tests behind the rows it carries.
pub(super) fn scroll_owner(
    descriptor: ComponentDescriptor,
    panel: DashboardScrollPanel,
    allocation: ComponentAllocationMeasurementContract,
) -> ComponentDescriptor {
    let index = DashboardScrollPanel::ALL
        .iter()
        .position(|candidate| *candidate == panel)
        .expect("every scroll panel is listed");
    descriptor
        .with_region_allocation(
            MosaicRegionKindId::new(panel.region()).unwrap(),
            panel.region_placement(),
        )
        .with_hit_test(ComponentHitTestContract::allocation_bounds(
            ComponentHitTestOrder::front_to_back(900 + index as u32),
            allocation,
        ))
}
