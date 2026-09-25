//! Ordinary application definition used by every production matrix row.

use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentSemanticTextContract, ComponentStateOwnership,
    ComponentViewportInset, SurfaceDescriptor, SurfaceId, SurfaceKind, SurfacePlacementClass,
    SurfaceStateClass, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenSource,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_native_platform::{
    UiNativeApplicationDefinition, UiNativeApplicationPreparation,
    UiNativeApplicationPreparationOutcome,
};

use super::case::{
    color, token, Phase5LocalityAxis, Phase5LocalityCase, BASE_TOKEN, ROOT, SURFACE, TARGET_TOKEN,
};
use super::timings::Phase5LocalityApplicationTimingRecorder;

pub(super) struct Phase5LocalityMatrixApplication {
    case: Phase5LocalityCase,
    presentation_async: worth_ui_query_binding::WorthUiPresentationAsyncInstallation,
    components: Box<[ComponentDescriptor]>,
    artifact: WorthUiRustAuthoredArtifactInput,
    timings: Phase5LocalityApplicationTimingRecorder,
}

impl Phase5LocalityMatrixApplication {
    pub(super) fn new(
        case: Phase5LocalityCase,
        presentation_async: worth_ui_query_binding::WorthUiPresentationAsyncInstallation,
    ) -> (Self, Phase5LocalityApplicationTimingRecorder) {
        let timings = Phase5LocalityApplicationTimingRecorder::default();
        let started = std::time::Instant::now();
        let components = std::iter::once(root_component())
            .chain((0..case.retained_paragraphs()).map(|index| text_component(case, index)))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let artifact = matrix_artifact(case);
        timings.record_fixture_materialization(started.elapsed());
        (
            Self {
                case,
                presentation_async,
                components,
                artifact,
                timings: timings.clone(),
            },
            timings,
        )
    }
}

impl UiNativeApplicationDefinition for Phase5LocalityMatrixApplication {
    fn prepare(
        self,
        mut preparation: UiNativeApplicationPreparation,
    ) -> UiNativeApplicationPreparationOutcome {
        let owner_started = std::time::Instant::now();
        let owner_result = preparation
            .install_presentation_async(self.presentation_async)
            .and_then(|()| preparation.install_frame_program(self.case.program()));
        self.timings
            .record_owner_installation(owner_started.elapsed());
        if let Err(cause) = owner_result {
            return preparation.deny(cause);
        }
        let builder_started = std::time::Instant::now();
        let result = (|| {
            let mut builder = preparation.builder();
            builder
                .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())?;
            builder.register_theme_token(matrix_token(BASE_TOKEN, "#f0f2f5"))?;
            builder.register_theme_token(matrix_token(TARGET_TOKEN, "#e53935"))?;
            for component in self.components {
                builder.register_component(component)?;
            }
            builder.register_surface(SurfaceDescriptor::new(
                SurfaceId::new(SURFACE).expect("matrix surface identity"),
                SurfaceKind::primary_content(),
                ComponentId::new(ROOT).expect("matrix root identity"),
                SurfacePlacementClass::primary_region(),
                SurfaceStateClass::ephemeral(),
            ))?;
            builder.with_rust_authored_input(self.artifact)
        })();
        self.timings
            .record_builder_registration(builder_started.elapsed());
        match result {
            Ok(()) => {
                let completion_started = std::time::Instant::now();
                let outcome = preparation.complete();
                self.timings
                    .record_application_completion(completion_started.elapsed());
                outcome
            }
            Err(cause) => preparation.deny(cause),
        }
    }
}

fn root_component() -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(ROOT).expect("matrix root identity"),
        ComponentPropSchema::named("phase5.matrix.root.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(ComponentAllocationMeasurementContract::fill_viewport())
}

fn text_component(case: Phase5LocalityCase, index: usize) -> ComponentDescriptor {
    let identity = case.component_identity(index);
    let measurement = if case.axis() == Phase5LocalityAxis::Width && index == case.target_index() {
        ComponentAllocationMeasurementContract::viewport_inset(ComponentViewportInset::symmetric(
            8, 8,
        ))
    } else {
        ComponentAllocationMeasurementContract::fixed_logical_size(144, 24)
            .expect("matrix fixed paragraph bounds")
    };
    let foreground = if index == case.target_index() {
        TARGET_TOKEN
    } else {
        BASE_TOKEN
    };
    ComponentDescriptor::new(
        ComponentId::new(identity).expect("matrix text identity"),
        ComponentPropSchema::named("phase5.matrix.text.props"),
        ComponentChildPolicy::text_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(measurement)
    .with_semantic_text(ComponentSemanticTextContract::body_default(
        token(foreground),
        u32::try_from(index + 1).expect("matrix layer order is bounded"),
    ))
}

fn matrix_token(identity: &str, value: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        token(identity),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        color(value),
    )
}

fn matrix_artifact(case: Phase5LocalityCase) -> WorthUiRustAuthoredArtifactInput {
    let mut module = WorthUiRustAuthoredArtifactInputModule::new("app/phase5_locality_matrix.wui")
        .with_token(BASE_TOKEN, "#f0f2f5")
        .with_token(TARGET_TOKEN, "#e53935")
        .with_component_authored_identity(ROOT, "phase5-matrix-root")
        .with_surface_authored_identity(SURFACE, "phase5-matrix-surface");
    for index in 0..case.retained_paragraphs() {
        module = module.with_component_authored_identity(
            case.component_identity(index),
            format!("phase5-matrix-text-{index}"),
        );
    }
    WorthUiRustAuthoredArtifactInput::from_modules([module])
}
