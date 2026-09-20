use worth_ui_host_contract::UiMountedScrollChromeMechanic;

pub(super) fn translate(
    mechanic: &UiMountedScrollChromeMechanic,
) -> super::UiHeadlessAppearanceMechanic {
    super::UiHeadlessAppearanceMechanic::ScrollChrome(*mechanic)
}
