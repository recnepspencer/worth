use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentSemanticTextContract, ComponentStateOwnership, SurfaceDescriptor,
    SurfaceId, SurfaceKind, SurfacePlacementClass, SurfaceStateClass, ThemeTokenDescriptor,
    ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue, UiThemeColor,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_native_platform::{
    UiNativeApplicationDefinition, UiNativeApplicationFrame, UiNativeApplicationPreparation,
    UiNativeApplicationPreparationOutcome, UiNativeApplicationProgram,
    UiNativeComponentPresenceChange, UiNativeComponentSemanticTextChange,
};

const FIRST: &str = "gate.d.text.first";
const SECOND: &str = "gate.d.text.second";
const BASELINE: &str = "gate.d.baseline";
const FIRST_AUTHORED: &str = "gate-d-first";
const SECOND_AUTHORED: &str = "gate-d-second";
const SURFACE: &str = "gate.d.surface";
const BASELINE_TOKEN: &str = "theme.gate_d.baseline";
const TEXT_TOKEN: &str = "theme.gate_d.text";
const FIRST_TEXT: &str = "CURRENT\u{2764}\u{FE0F}A";
const SECOND_TEXT: &str = "CURRENT\u{2764}\u{FE0F}B";

pub(crate) struct PlatformPulseNativeGateDApplication {
    presentation_async: worth_ui::facade::query_binding::WorthUiPresentationAsyncInstallation,
}

impl PlatformPulseNativeGateDApplication {
    pub(crate) fn new(
        presentation_async: worth_ui::facade::query_binding::WorthUiPresentationAsyncInstallation,
    ) -> Self {
        Self { presentation_async }
    }
}

impl UiNativeApplicationDefinition for PlatformPulseNativeGateDApplication {
    fn prepare(
        self,
        mut preparation: UiNativeApplicationPreparation,
    ) -> UiNativeApplicationPreparationOutcome {
        let result = (|| {
            preparation.install_presentation_async(self.presentation_async)?;
            preparation.install_frame_program(gate_d_program())?;
            let mut builder = preparation.builder();
            builder
                .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())?;
            builder.register_theme_token(baseline_token())?;
            builder.register_theme_token(text_token())?;
            builder.register_component(baseline_component())?;
            builder.register_component(text_component(FIRST, 0))?;
            builder.register_component(text_component(SECOND, 0))?;
            builder.register_surface(SurfaceDescriptor::new(
                SurfaceId::new(SURFACE).expect("Gate D surface identity"),
                SurfaceKind::primary_content(),
                ComponentId::new(BASELINE).expect("Gate D root component"),
                SurfacePlacementClass::primary_region(),
                SurfaceStateClass::ephemeral(),
            ))?;
            let module = WorthUiRustAuthoredArtifactInputModule::new("app/gate_d_text.wui")
                .with_token(BASELINE_TOKEN, "#1f2328")
                .with_token(TEXT_TOKEN, "#ffffff")
                .with_component_authored_identity(BASELINE, "gate-d-baseline")
                .with_component_authored_identity(FIRST, FIRST_AUTHORED)
                .with_component_authored_identity(SECOND, SECOND_AUTHORED)
                .with_surface_authored_identity(SURFACE, "gate-d-surface");
            builder
                .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        })();
        match result {
            Ok(()) => preparation.complete(),
            Err(cause) => preparation.deny(cause),
        }
    }
}

fn gate_d_program() -> UiNativeApplicationProgram {
    UiNativeApplicationProgram::new([
        UiNativeApplicationFrame::with_semantic_text([text_change(FIRST), text_change(SECOND)])
            .expect("Gate D initial text frame"),
        UiNativeApplicationFrame::with_component_presence([presence_change(FIRST, false)])
            .expect("Gate D first-owner release frame"),
        UiNativeApplicationFrame::with_component_presence([presence_change(SECOND, false)])
            .expect("Gate D last-owner release frame"),
        UiNativeApplicationFrame::present_current(),
    ])
    .expect("Gate D frame program is bounded")
}

fn baseline_component() -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(BASELINE).expect("Gate D baseline identity"),
        ComponentPropSchema::named("gate.d.baseline.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(ComponentAllocationMeasurementContract::fill_viewport())
    .with_surface_paint_order(0)
}

fn text_component(identity: &str, paint_order: u32) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(identity).expect("Gate D component identity"),
        ComponentPropSchema::named(format!("{identity}.props")),
        ComponentChildPolicy::text_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(ComponentAllocationMeasurementContract::fill_viewport())
    .with_semantic_text(ComponentSemanticTextContract::body_default(
        ThemeTokenId::new(TEXT_TOKEN).expect("Gate D text token"),
        paint_order,
    ))
}

fn text_token() -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(TEXT_TOKEN).expect("Gate D text token identity"),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse("#ffffff").expect("Gate D text color")),
    )
}

fn baseline_token() -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(BASELINE_TOKEN).expect("Gate D baseline token identity"),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse("#1f2328").expect("Gate D baseline color")),
    )
}

fn text_change(identity: &str) -> UiNativeComponentSemanticTextChange {
    let text = match identity {
        FIRST => FIRST_TEXT,
        SECOND => SECOND_TEXT,
        _ => unreachable!("Gate D text identity is closed"),
    };
    UiNativeComponentSemanticTextChange::new(format!("component:{identity}"), text)
        .expect("Gate D semantic text change")
}

fn presence_change(identity: &str, present: bool) -> UiNativeComponentPresenceChange {
    UiNativeComponentPresenceChange::new(format!("component:{identity}"), present)
        .expect("Gate D component presence change")
}
