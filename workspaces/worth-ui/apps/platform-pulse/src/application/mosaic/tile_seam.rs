//! The Service tile owns its shared exterior edge with the Native tile.
use super::{region_id, PlatformPulseMosaicRegion};
use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::declaration::{
    MosaicSeamPaintContract, MosaicSeamPaintOwner, MosaicSharedEdge,
};

pub(super) fn register(
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    let service = region_id(PlatformPulseMosaicRegion::ServiceTile);
    let native = region_id(PlatformPulseMosaicRegion::NativeTile);
    let edge = MosaicSharedEdge::new(service.clone(), native.clone())
        .expect("distinct Pulse tile regions share one edge");
    let contract = MosaicSeamPaintContract::admit(
        PlatformPulseMosaicRegion::ALL.map(region_id),
        [edge.clone()],
        [MosaicSeamPaintOwner::new(edge, service).expect("Service owns the tile seam")],
        [],
    )
    .expect("Pulse tile seam has one declared owner");
    builder
        .register_mosaic_seam_paint_contract(contract)
        .expect("Pulse tile seam references registered region kinds")
}
