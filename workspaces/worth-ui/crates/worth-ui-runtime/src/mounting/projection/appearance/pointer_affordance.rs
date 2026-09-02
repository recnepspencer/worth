use worth_ui_host_contract::UiMountedAppearanceMechanic;

use super::fact::UiMountedAppearanceNodeInput;
use super::UiMountedAppearanceLoweringDenial;

pub(super) fn lower(
    input: &UiMountedAppearanceNodeInput,
) -> Result<Option<UiMountedAppearanceMechanic>, UiMountedAppearanceLoweringDenial> {
    let Some(pointer) = input.pointer else {
        return Ok(None);
    };
    if pointer.target != input.node_receipt.mounted_instance() {
        return Err(UiMountedAppearanceLoweringDenial::PointerTargetMismatch);
    }
    if pointer.surface != input.semantic_surface {
        return Err(UiMountedAppearanceLoweringDenial::PointerSurfaceMismatch);
    }
    Ok(Some(UiMountedAppearanceMechanic::Pointer(
        worth_ui_host_contract::UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
            pointer.pointer,
            pointer.surface,
            pointer.target,
            pointer.family,
        ),
    )))
}
