mod appearance;
mod product_structure;
mod product_structure_geometry;

use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::declaration::{
    ThemeColorValue, ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId,
    ThemeTokenSource, ThemeTokenValue,
};
use worth_ui::facade::inspection::{
    UiVisualInspectionByteBudget, UiVisualInspectionCapacity, UiVisualInspectionPolicy,
    UiVisualInspectionRegionCapacity,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES;

pub(super) use appearance::register as register_appearance;
pub(super) use appearance::PlatformPulseAppearanceRegistrationDenial;
pub(super) use product_structure::register_structure;

const FILL_TOKEN: &str = "theme.platform_pulse.fill";
const IDENTITY_TARGET_FILL_TOKEN: &str = "theme.platform_pulse.identity_target_fill";
const CONFIRMATION_FILL_TOKEN: &str = "theme.platform_pulse.confirmation_fill";
const BLUE_TOKEN: &str = "theme.platform_pulse.blue";
const GREEN_TOKEN: &str = "theme.platform_pulse.green";
const YELLOW_TOKEN: &str = "theme.platform_pulse.yellow";
const WHITE_TOKEN: &str = "theme.platform_pulse.white";
const PURPLE_TOKEN: &str = "theme.platform_pulse.purple";
const TEXT_TOKEN: &str = "theme.platform_pulse.projected_status.text";
const PLATFORM_PULSE_RETAINED_PIXEL_BYTES: u64 = 2 * PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES;
// The authored Portal surface adds qualified child mechanics to the retained
// structural receipt. Keep the declaration bounded while covering both the
// 960-by-600 and resized product compositions.
const PLATFORM_PULSE_STRUCTURAL_BYTES_PER_RECEIPT: u64 = 512 << 10;
const PLATFORM_PULSE_RETAINED_STRUCTURAL_BYTES: u64 =
    2 * PLATFORM_PULSE_STRUCTURAL_BYTES_PER_RECEIPT;

pub(super) fn register_theme_tokens(
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    use worth_ui_platform_pulse::product_world::PlatformPulsePaletteRole;

    let builder = PlatformPulsePaletteRole::ALL
        .into_iter()
        .fold(builder, |builder, role| {
            builder
                .register_theme_token(role.token_descriptor())
                .register_theme_token(role.source_alias_descriptor())
        });
    builder
        .register_theme_token(ThemeTokenDescriptor::define(
            token_id(WHITE_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(
                ThemeColorValue::hex("#ffffff").expect("valid Pulse text color"),
            ),
        ))
        .register_theme_token(ThemeTokenDescriptor::define(
            token_id(YELLOW_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(
                ThemeColorValue::hex("#f2cc60").expect("valid Pulse target color"),
            ),
        ))
        .register_theme_token(ThemeTokenDescriptor::define(
            token_id(PURPLE_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(
                ThemeColorValue::hex("#6e40c9").expect("valid Pulse confirmation color"),
            ),
        ))
        .register_theme_token(ThemeTokenDescriptor::define(
            token_id(BLUE_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(ThemeColorValue::hex("#2f81f7").expect("valid Pulse blue")),
        ))
        .register_theme_token(ThemeTokenDescriptor::define(
            token_id(GREEN_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(ThemeColorValue::hex("#3fb950").expect("valid Pulse green")),
        ))
        .register_theme_token(ThemeTokenDescriptor::alias(
            token_id(IDENTITY_TARGET_FILL_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(PlatformPulsePaletteRole::PrincipalAccent.token_id()),
        ))
        .register_theme_token(ThemeTokenDescriptor::alias(
            token_id(CONFIRMATION_FILL_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(PlatformPulsePaletteRole::Caution.token_id()),
        ))
        .register_theme_token(ThemeTokenDescriptor::alias(
            token_id(FILL_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(token_id(BLUE_TOKEN)),
        ))
        .register_theme_token(ThemeTokenDescriptor::alias(
            token_id(TEXT_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(PlatformPulsePaletteRole::Caution.token_id()),
        ))
}

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

fn token_id(text: &str) -> ThemeTokenId {
    ThemeTokenId::new(text).expect("valid Pulse theme token id")
}
