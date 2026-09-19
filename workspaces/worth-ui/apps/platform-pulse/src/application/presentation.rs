mod appearance;
mod fonts;
mod product_structure;
pub(super) use fonts::PulseFonts;

use worth_ui::facade::inspection::{
    UiVisualInspectionByteBudget, UiVisualInspectionCapacity, UiVisualInspectionPolicy,
    UiVisualInspectionRegionCapacity,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES;

pub(super) use appearance::register as register_appearance;
pub(super) use appearance::PlatformPulseAppearanceRegistrationDenial;
pub(super) use product_structure::register_structure;

const PLATFORM_PULSE_RETAINED_PIXEL_BYTES: u64 = 2 * PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES;
// The full dashboard retains text glyphs, vector mechanics and both overlays.
// Inspection remains bounded to two snapshots of at most 8 MiB each.
const PLATFORM_PULSE_STRUCTURAL_BYTES_PER_RECEIPT: u64 = 8 << 20;
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
