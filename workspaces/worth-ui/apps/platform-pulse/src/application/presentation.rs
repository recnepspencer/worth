mod appearance;
mod product_structure;
mod product_structure_geometry;

use worth_ui::facade::inspection::{
    UiVisualInspectionByteBudget, UiVisualInspectionCapacity, UiVisualInspectionPolicy,
    UiVisualInspectionRegionCapacity,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES;

pub(super) use appearance::register as register_appearance;
pub(super) use appearance::PlatformPulseAppearanceRegistrationDenial;
pub(super) use product_structure::register_structure;

const PLATFORM_PULSE_RETAINED_PIXEL_BYTES: u64 = 2 * PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES;
// Active-dialog snapshots include mounted geometry, identity traces, and paint
// indices (the first dialog already reserves 528,330 bytes). Bound the full
// two-dialog composition to 1 MiB per receipt and two retained receipts.
const PLATFORM_PULSE_STRUCTURAL_BYTES_PER_RECEIPT: u64 = 1 << 20;
const PLATFORM_PULSE_RETAINED_STRUCTURAL_BYTES: u64 =
    2 * PLATFORM_PULSE_STRUCTURAL_BYTES_PER_RECEIPT;

pub(super) fn visual_inspection_policy() -> UiVisualInspectionPolicy {
    UiVisualInspectionPolicy::bounded(
        worth_ui::facade::inspection::UiVisualInspectionDisclosure::local_development_unredacted(),
        UiVisualInspectionCapacity::bounded(2, 8, 16),
        UiVisualInspectionRegionCapacity::bounded(65_536, 65_536),
        UiVisualInspectionByteBudget::bounded(
            PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES,
            PLATFORM_PULSE_RETAINED_PIXEL_BYTES,
            PLATFORM_PULSE_STRUCTURAL_BYTES_PER_RECEIPT,
            PLATFORM_PULSE_RETAINED_STRUCTURAL_BYTES,
        ),
    )
    .expect("the permanent Pulse declares a valid bounded visual policy")
}
