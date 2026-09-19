use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, ThemeTokenDescriptor, ThemeTokenFamily,
    ThemeTokenId, ThemeTokenSource, ThemeTokenValue, UiThemeColor,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_native_platform::{
    UiNativeApplicationDefinition, UiNativeApplicationFrame, UiNativeApplicationPreparation,
    UiNativeApplicationPreparationOutcome, UiNativeApplicationProgram,
    UiNativeComponentPresenceChange,
};

const RECTANGLE_COUNT: usize = 2_048;

pub(crate) struct PlatformPulseNativePhase3Application;

impl UiNativeApplicationDefinition for PlatformPulseNativePhase3Application {
    fn prepare(
        self,
        mut preparation: UiNativeApplicationPreparation,
    ) -> UiNativeApplicationPreparationOutcome {
        let result = (|| {
            preparation.install_frame_program(phase3_program())?;
            let mut builder = preparation.builder();
            builder
                .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())?;
            let mut module = WorthUiRustAuthoredArtifactInputModule::new("app/phase3.wui");
            for index in 0..RECTANGLE_COUNT {
                let component = component_identity(index);
                let token = token_identity(index);
                builder.register_theme_token(theme_token(&token, color(index)))?;
                builder.register_component(component_descriptor(&component, index))?;
                module = module
                    .with_token(&token, color(index))
                    .with_component_authored_identity(
                        &component,
                        format!("platform-pulse-native-phase3-{index:04}"),
                    );
            }
            builder
                .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        })();
        match result {
            Ok(()) => preparation.complete(),
            Err(cause) => preparation.deny(cause),
        }
    }
}

fn phase3_program() -> UiNativeApplicationProgram {
    let mut frames = vec![
        UiNativeApplicationFrame::present_current(),
        UiNativeApplicationFrame::present_current(),
    ];
    for count in [1, 1_024, 2_048] {
        let start = RECTANGLE_COUNT - count;
        frames.push(presence_frame((start..RECTANGLE_COUNT).rev(), false));
        frames.push(presence_frame(start..RECTANGLE_COUNT, true));
    }
    UiNativeApplicationProgram::new(frames)
        .expect("the fixed Phase 3 program is bounded")
        .remain_open_until_external_close()
}

fn presence_frame(
    indices: impl IntoIterator<Item = usize>,
    present: bool,
) -> UiNativeApplicationFrame {
    let changes = indices.into_iter().map(|index| {
        UiNativeComponentPresenceChange::new(
            format!("component:{}", component_identity(index)),
            present,
        )
        .expect("Phase 3 component presence identity")
    });
    UiNativeApplicationFrame::with_component_presence(changes)
        .expect("Phase 3 frame stays within the qualified command capacity")
}

fn component_descriptor(component: &str, index: usize) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(component).expect("phase3 component identity"),
        ComponentPropSchema::named(format!("{component}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(ComponentAllocationMeasurementContract::fill_viewport())
    .with_surface_paint_order(index as u32)
}

fn theme_token(identity: &str, value: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(identity).expect("phase3 token identity"),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse(value).expect("phase3 qualified color")),
    )
}

fn component_identity(index: usize) -> String {
    format!("platform.pulse.phase3.rect_{index:04}")
}

fn token_identity(index: usize) -> String {
    format!("theme.platform_pulse.phase3.color_{index:04}")
}

fn color(index: usize) -> &'static str {
    match index {
        1_023 | 2_046 => "#f2cc60",
        _ => "#2f81f7",
    }
}
