use worth_ui_host_contract::{
    UiMountedRgba8, UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity,
    UiTextOriginalRange,
};

use super::super::UiMountedProjectionDenial;

mod materialization;

#[cfg(test)]
#[path = "formatting_tests.rs"]
mod tests;

#[derive(Clone, PartialEq)]
pub(in crate::mounting::projection) struct UiMountedSemanticTextFormattingSeed {
    default: UiMountedSemanticTextDefault,
    scalar_spans: Box<[UiMountedSemanticTextResolvedSpan]>,
    layer_semantic_order: u32,
}

#[derive(Clone, PartialEq)]
pub(in crate::mounting::projection) struct UiMountedSemanticTextDefault {
    color: UiMountedRgba8,
    style: Option<worth_ui_text::UiTextStyle>,
    line_height_millipoints: Option<u32>,
    paint_identity: UiMountedTextPaintSpanIdentity,
}

#[derive(Clone, PartialEq)]
pub(in crate::mounting::projection) struct UiMountedSemanticTextResolvedSpan {
    original_range: UiTextOriginalRange,
    color: UiMountedRgba8,
    style: worth_ui_text::UiTextStyle,
    paint_identity: UiMountedTextPaintSpanIdentity,
    appearance_foreground: bool,
}

#[derive(Clone, Copy)]
pub(in crate::mounting::projection) enum UiMountedSemanticTextRowFormatting<'a> {
    Default(&'a UiMountedSemanticTextDefault),
    ScalarSpans(&'a [UiMountedSemanticTextResolvedSpan], Option<u32>),
}

pub(in crate::mounting::projection) fn lower_semantic_text_formatting(
    plan: super::super::super::UiMountedPlanProjectionSource<'_>,
    theme_values: &crate::mounting::UiMountedThemeValueSource,
    _graph_node: crate::graph::UiGraphNodeIdentity,
    plan_index: Option<u32>,
    input: Option<&crate::mounting::UiMountedSemanticTextContent>,
    predecessor: Option<&super::UiMountedSemanticTextSeed>,
    theme_value_changed: bool,
) -> Result<Option<UiMountedSemanticTextFormattingSeed>, UiMountedProjectionDenial> {
    if let Some(crate::mounting::UiMountedSemanticTextContent::Scalar(input)) = input {
        if let Some(directive) = input.formatting() {
            return lower_directive(directive).map(Some);
        }
    }
    if let Some(predecessor) = predecessor.filter(|_| !theme_value_changed) {
        return Ok(Some(predecessor.formatting().clone()));
    }
    let Some(plan_index) = plan_index else {
        return Ok(None);
    };
    let Some(meaning) = plan.ordinary_meaning(plan_index) else {
        return Ok(None);
    };
    let crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::Component(
        component,
    ) = meaning.as_ref()
    else {
        return Ok(None);
    };
    let Some(contract) = component.semantic_text_contract() else {
        return Ok(None);
    };
    let default = UiMountedSemanticTextDefault {
        color: resolve_color(plan, theme_values, contract.theme_token())?,
        style: contract.style().cloned(),
        line_height_millipoints: contract.line_height_millipoints(),
        paint_identity: UiMountedTextPaintSpanIdentity::from_runtime_mounting(
            contract.default_paint_identity(),
        ),
    };
    let scalar_spans = contract
        .scalar_spans()
        .iter()
        .map(|span| {
            Ok(UiMountedSemanticTextResolvedSpan {
                original_range: span.original_range(),
                appearance_foreground: span.uses_appearance_foreground(),
                color: resolve_color(plan, theme_values, span.foreground_token())?,
                style: span.style().clone(),
                paint_identity: UiMountedTextPaintSpanIdentity::from_runtime_mounting(
                    span.paint_identity(),
                ),
            })
        })
        .collect::<Result<Vec<_>, UiMountedProjectionDenial>>()?;
    Ok(Some(UiMountedSemanticTextFormattingSeed {
        default,
        scalar_spans: scalar_spans.into_boxed_slice(),
        layer_semantic_order: contract.layer_semantic_order(),
    }))
}

fn lower_directive(
    directive: &crate::mounting::UiMountedSemanticTextFormattingDirective,
) -> Result<UiMountedSemanticTextFormattingSeed, UiMountedProjectionDenial> {
    let contract = directive.contract();
    let default = UiMountedSemanticTextDefault {
        color: resolve_directive_color(directive, contract.theme_token())?,
        style: contract.style().cloned(),
        line_height_millipoints: contract.line_height_millipoints(),
        paint_identity: UiMountedTextPaintSpanIdentity::from_runtime_mounting(
            contract.default_paint_identity(),
        ),
    };
    let scalar_spans = contract
        .scalar_spans()
        .iter()
        .map(|span| {
            Ok(UiMountedSemanticTextResolvedSpan {
                original_range: span.original_range(),
                appearance_foreground: span.uses_appearance_foreground(),
                color: resolve_directive_color(directive, span.foreground_token())?,
                style: span.style().clone(),
                paint_identity: UiMountedTextPaintSpanIdentity::from_runtime_mounting(
                    span.paint_identity(),
                ),
            })
        })
        .collect::<Result<Vec<_>, UiMountedProjectionDenial>>()?;
    Ok(UiMountedSemanticTextFormattingSeed {
        default,
        scalar_spans: scalar_spans.into_boxed_slice(),
        layer_semantic_order: contract.layer_semantic_order(),
    })
}

fn resolve_directive_color(
    directive: &crate::mounting::UiMountedSemanticTextFormattingDirective,
    token: &crate::capability::ThemeTokenId,
) -> Result<UiMountedRgba8, UiMountedProjectionDenial> {
    let Some(crate::capability::ThemeTokenValue::Color(color)) = directive.token_value(token)
    else {
        return Err(UiMountedProjectionDenial::MissingSemanticTextToken);
    };
    super::super::static_paint::parse_rgba(color.as_str())
        .map_err(|_| UiMountedProjectionDenial::InvalidSemanticTextColor)
}

fn resolve_color(
    plan: super::super::super::UiMountedPlanProjectionSource<'_>,
    theme_values: &crate::mounting::UiMountedThemeValueSource,
    token_id: &crate::capability::ThemeTokenId,
) -> Result<UiMountedRgba8, UiMountedProjectionDenial> {
    if let Some(value) = theme_values.current_value(token_id) {
        let crate::capability::ThemeTokenValue::Color(color) = value else {
            return Err(UiMountedProjectionDenial::MissingSemanticTextToken);
        };
        return super::super::static_paint::parse_rgba(color.as_str())
            .map_err(|_| UiMountedProjectionDenial::InvalidSemanticTextColor);
    }
    if !theme_values.uses_frozen_plan() {
        return Err(UiMountedProjectionDenial::MissingSemanticTextToken);
    }
    let Some((_token_plan_index, token_meaning)) = plan
        .semantic_text_token(token_id)
        .map_err(|_| UiMountedProjectionDenial::AmbiguousSemanticTextToken)?
    else {
        return Err(UiMountedProjectionDenial::MissingSemanticTextToken);
    };
    let crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::Token(token) =
        token_meaning.as_ref()
    else {
        return Err(UiMountedProjectionDenial::ForeignSemanticTextToken);
    };
    let color = token
        .resolved_color_text()
        .ok_or(UiMountedProjectionDenial::MissingSemanticTextColor)?;
    super::super::static_paint::parse_rgba(color)
        .map_err(|_| UiMountedProjectionDenial::InvalidSemanticTextColor)
}

impl UiMountedSemanticTextFormattingSeed {
    pub(in crate::mounting::projection) fn appearance_foreground_spans(
        &self,
    ) -> Box<[UiMountedTextPaintSpanIdentity]> {
        self.scalar_spans
            .iter()
            .filter(|span| span.appearance_foreground)
            .map(|span| span.paint_identity)
            .collect()
    }

    pub(in crate::mounting::projection) const fn layer_semantic_order(&self) -> u32 {
        self.layer_semantic_order
    }

    pub(in crate::mounting::projection) const fn default_row(
        &self,
    ) -> UiMountedSemanticTextRowFormatting<'_> {
        UiMountedSemanticTextRowFormatting::Default(&self.default)
    }

    pub(in crate::mounting::projection) fn scalar_value_row(
        &self,
    ) -> UiMountedSemanticTextRowFormatting<'_> {
        if self.scalar_spans.is_empty() {
            self.default_row()
        } else {
            UiMountedSemanticTextRowFormatting::ScalarSpans(
                &self.scalar_spans,
                self.default.line_height_millipoints,
            )
        }
    }

    pub(in crate::mounting::projection) fn same_layout_as(&self, other: &Self) -> bool {
        self.default.style == other.default.style
            && self.default.line_height_millipoints == other.default.line_height_millipoints
            && self.scalar_spans.len() == other.scalar_spans.len()
            && self
                .scalar_spans
                .iter()
                .zip(other.scalar_spans.iter())
                .all(|(left, right)| {
                    left.original_range == right.original_range && left.style == right.style
                })
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn body_default_for_test() -> Self {
        Self::body_default_with_color_for_test(UiMountedRgba8::new(255, 255, 255, 255))
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn body_default_with_color_for_test(
        color: UiMountedRgba8,
    ) -> Self {
        Self::body_default_with_color_and_layer_for_test(color, 0)
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn body_default_with_color_and_layer_for_test(
        color: UiMountedRgba8,
        layer_semantic_order: u32,
    ) -> Self {
        Self {
            default: UiMountedSemanticTextDefault {
                color,
                style: None,
                line_height_millipoints: None,
                paint_identity: UiMountedTextPaintSpanIdentity::from_runtime_mounting([1; 32]),
            },
            scalar_spans: Box::new([]),
            layer_semantic_order,
        }
    }
}
