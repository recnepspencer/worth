use std::sync::Arc;

use worth_ui_host_contract::UiMountedSemanticTextMechanic;

use super::super::UiMountedProjectionDenial;
use super::completion::UiMountedSemanticTextCompletionContext;

pub(super) fn qualify_layout(
    context: &UiMountedSemanticTextCompletionContext<'_>,
    source: &Arc<str>,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    formatting: super::formatting::UiMountedSemanticTextRowFormatting<'_>,
) -> Result<UiMountedTextQualification, UiMountedProjectionDenial> {
    let constraints = paragraph_constraints(bounds, formatting)?;
    let (styles, foregrounds) = formatting.materialize(source, &constraints)?;
    let input = worth_ui_text::UiTextParagraphAdmissionInput {
        source: Arc::clone(source),
        constraints,
        profile_generation: super::current_text_profile_generation(),
        font_collection_generation: context.font_collection.generation(),
        text_scale_generation: worth_ui_host_contract::UiTextScaleGeneration::new(1)
            .expect("initial text scale generation"),
        styles,
    };
    let request = worth_ui_text::UiQualifiedTextLayoutRequest::new(
        input,
        Arc::clone(context.font_collection),
    );
    let (layout, shaped) = context
        .qualification_cache
        .qualify(request)
        .map_err(UiMountedProjectionDenial::SemanticTextQualification)?;
    Ok(UiMountedTextQualification {
        layout,
        foregrounds,
        shaped,
    })
}

pub(super) fn paragraph_constraints(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    formatting: super::formatting::UiMountedSemanticTextRowFormatting<'_>,
) -> Result<worth_ui_text::UiTextParagraphConstraints, UiMountedProjectionDenial> {
    let width = logical_millipoints(bounds.width())?;
    // The box height clips the shaped lines; it never decides how many there are.
    logical_millipoints(bounds.height())?;
    let flow = formatting.flow();
    worth_ui_text::UiTextParagraphConstraints::new(worth_ui_text::UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: worth_ui_text::UiTextBaseDirection::Auto,
        wrap: flow.wrap(),
        alignment: formatting.alignment(),
        overflow: flow.overflow(),
        font_size_millipoints: 14_000,
        width_millipoints: width,
        line_height_millipoints: formatting.line_height_millipoints().unwrap_or(18_000),
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines: flow.line_limit(),
    })
    .ok_or(UiMountedProjectionDenial::SemanticTextShapeMismatch)
}

pub(super) struct UiMountedTextQualification {
    layout: Arc<worth_ui_text::UiQualifiedTextLayout>,
    foregrounds: Arc<[worth_ui_host_contract::UiMountedTextForegroundSpan]>,
    shaped: bool,
}

impl UiMountedTextQualification {
    pub(super) fn layout(&self) -> &Arc<worth_ui_text::UiQualifiedTextLayout> {
        &self.layout
    }

    pub(super) fn foregrounds(
        &self,
    ) -> &Arc<[worth_ui_host_contract::UiMountedTextForegroundSpan]> {
        &self.foregrounds
    }

    /// Completes the mechanic, recording shaping cost only when this
    /// qualification shaped the text rather than reusing a layout.
    pub(super) fn complete(
        &self,
        input: worth_ui_host_contract::UiMountedSemanticTextCompletionInput<'_>,
    ) -> Result<
        worth_ui_host_contract::UiMountedSemanticTextMechanic,
        worth_ui_host_contract::UiMountedSemanticTextCompletionDenial,
    > {
        if self.shaped {
            UiMountedSemanticTextMechanic::complete_from_runtime_mounting(input)
        } else {
            UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(input)
        }
    }
}

fn logical_millipoints(value: f32) -> Result<u32, UiMountedProjectionDenial> {
    let scaled = f64::from(value) * 1_000.0;
    if !scaled.is_finite() || scaled <= 0.0 || scaled > f64::from(u32::MAX) {
        return Err(UiMountedProjectionDenial::SemanticTextShapeMismatch);
    }
    Ok(scaled.ceil() as u32)
}
