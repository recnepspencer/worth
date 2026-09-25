use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentDescriptor, ComponentHitTestContract,
    ComponentHitTestOrder, MosaicRegionKindId,
};
use worth_ui_platform_pulse::product_world::DashboardScrollOwner;

/// Makes a content container the owner that scrolls it: it places its
/// region and hit-tests behind the content it carries, a list in front of
/// the page that carries the list.
pub(super) fn scroll_owner(
    descriptor: ComponentDescriptor,
    owner: DashboardScrollOwner,
    allocation: ComponentAllocationMeasurementContract,
) -> ComponentDescriptor {
    let index = DashboardScrollOwner::ALL
        .iter()
        .position(|candidate| *candidate == owner)
        .expect("every scroll owner is listed");
    descriptor
        .with_region_allocation(
            MosaicRegionKindId::new(owner.region()).unwrap(),
            owner.region_placement(),
        )
        .with_hit_test(ComponentHitTestContract::allocation_bounds(
            ComponentHitTestOrder::front_to_back(900 + index as u32),
            allocation,
        ))
}
