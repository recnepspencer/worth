use worth_ui_dsl::UiMosaicRegionDeclarationIdentity;
use worth_ui_host_contract::UiMountedInstanceIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedMosaicClipBinding {
    Viewport,
    Region {
        owner: UiMountedInstanceIdentity,
        declaration: UiMosaicRegionDeclarationIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedScrollClipBinding {
    Viewport,
    Occurrence(UiMountedInstanceIdentity),
    Region {
        owner: UiMountedInstanceIdentity,
        declaration: UiMosaicRegionDeclarationIdentity,
    },
}
