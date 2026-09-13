use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, ComponentViewportInset, ThemeTokenDescriptor,
    ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue, UiThemeColor,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_native_platform::{
    UiNativeApplicationDefinition, UiNativeApplicationFrame, UiNativeApplicationPreparation,
    UiNativeApplicationPreparationOutcome, UiNativeApplicationProgram,
};

const COMPONENT: &str = "platform.pulse.native_seed.rectangle";
const TOKEN: &str = "theme.platform_pulse.native_seed.blue";
mod appearance;

/// Phase 2's text-free public-composition seed.
///
/// It proves the first real native vertical without pretending that the later
/// text, Query, intent, and cumulative parity phases already exist.
pub struct PlatformPulseNativeSeedApplication {
    program: NativeSeedProgram,
}

#[derive(Clone, Copy)]
enum NativeSeedProgram {
    Ordinary,
    CaptureInitial,
    CaptureAfterSurfaceSuccessor,
}

impl PlatformPulseNativeSeedApplication {
    pub const fn new() -> Self {
        Self {
            program: NativeSeedProgram::Ordinary,
        }
    }

    pub const fn with_presented_source_capture(mut self) -> Self {
        self.program = NativeSeedProgram::CaptureInitial;
        self
    }

    pub const fn with_surface_successor_capture(mut self) -> Self {
        self.program = NativeSeedProgram::CaptureAfterSurfaceSuccessor;
        self
    }
}

impl Default for PlatformPulseNativeSeedApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNativeApplicationDefinition for PlatformPulseNativeSeedApplication {
    fn prepare(
        self,
        mut preparation: UiNativeApplicationPreparation,
    ) -> UiNativeApplicationPreparationOutcome {
        let result = (|| {
            let role = appearance::role();
            let mut builder = worth_ui::facade::app::WorthUi::app()
                .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
                .register_theme_token(theme_token())
                .register_component(
                    component()
                        .with_appearance_aspect_contract(role.aspect_contract().clone())
                        .expect("native seed component accepts its background role"),
                )
                .register_appearance_role(role.clone())
                .expect("native seed role is unique")
                .register_appearance_theme_bundle(appearance::theme())
                .expect("native seed theme is complete")
                .with_rust_authored_input(authored_input(role));
            if !matches!(self.program, NativeSeedProgram::Ordinary) {
                builder = builder.with_visual_inspection_policy(visual_inspection_policy());
            }
            preparation.install_application_composition(builder)?;
            let program = match self.program {
                NativeSeedProgram::Ordinary => UiNativeApplicationProgram::single_frame(),
                NativeSeedProgram::CaptureInitial => {
                    UiNativeApplicationProgram::new([UiNativeApplicationFrame::present_current()
                        .capture_presented_source_pixels()])
                    .expect("the seed admits one bounded capture")
                }
                NativeSeedProgram::CaptureAfterSurfaceSuccessor => {
                    UiNativeApplicationProgram::new([
                        UiNativeApplicationFrame::present_current(),
                        UiNativeApplicationFrame::present_current()
                            .after_host_surface_basis_successor()
                            .capture_presented_source_pixels(),
                        UiNativeApplicationFrame::switch_theme(
                            worth_ui::facade::appearance::UiThemeDefinitionIdentity::new(
                                appearance::GREEN,
                            )
                            .expect("native seed Green theme is declared"),
                        )
                        .after_host_surface_basis_successor(),
                    ])
                    .expect("the seed admits one capture across two surface successors")
                }
            };
            preparation.install_frame_program(program.remain_open_until_external_close())
        })();
        match result {
            Ok(()) => preparation.complete(),
            Err(cause) => preparation.deny(cause),
        }
    }
}

fn visual_inspection_policy() -> worth_ui::facade::inspection::UiVisualInspectionPolicy {
    worth_ui::facade::inspection::UiVisualInspectionPolicy::bounded(
        worth_ui::facade::inspection::UiVisualInspectionDisclosure::local_development_unredacted(),
        worth_ui::facade::inspection::UiVisualInspectionCapacity::bounded(1, 1, 1),
        worth_ui::facade::inspection::UiVisualInspectionRegionCapacity::bounded(4, 4),
        worth_ui::facade::inspection::UiVisualInspectionByteBudget::bounded(
            16 * 1024 * 1024,
            16 * 1024 * 1024,
            64 * 1024,
            64 * 1024,
        ),
    )
    .expect("the seed declares a bounded visual inspection policy")
}

fn theme_token() -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(TOKEN).expect("valid native seed token"),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse("#2f81f7").expect("qualified blue")),
    )
}

fn component() -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(COMPONENT).expect("valid native seed component"),
        ComponentPropSchema::named("platform.pulse.native_seed.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(ComponentAllocationMeasurementContract::viewport_inset(
        ComponentViewportInset::symmetric(16, 12),
    ))
    .with_surface_paint_order(0)
}

fn authored_input(
    role: worth_ui::facade::appearance::UiAppearanceRoleDeclaration,
) -> WorthUiRustAuthoredArtifactInput {
    let attachment = worth_ui::facade::appearance::UiAppearanceRoleAttachmentDeclaration::new(
        role.role().clone(),
        role.revision(),
    );
    WorthUiRustAuthoredArtifactInput::from_modules([WorthUiRustAuthoredArtifactInputModule::new(
        "app/native_seed.wui",
    )
    .with_token(TOKEN, "#2f81f7")
    .with_component_authored_identity(COMPONENT, "platform-pulse-native-seed")
    .with_appearance_role(role)
    .with_component_appearance_role(COMPONENT, attachment)
    .expect("native seed component has one role attachment")])
}
