use super::*;

impl UiMountedSemanticTextRowFormatting<'_> {
    pub(in crate::mounting::projection::semantic_text) const fn line_height_millipoints(
        self,
    ) -> Option<u32> {
        match self {
            Self::Default(default) => default.line_height_millipoints,
            Self::ScalarSpans(_, line_height_millipoints) => line_height_millipoints,
        }
    }

    pub(in crate::mounting::projection::semantic_text) fn materialize(
        self,
        source: &str,
        constraints: &worth_ui_text::UiTextParagraphConstraints,
    ) -> Result<
        (
            Box<[worth_ui_text::UiTextStyleSpan]>,
            std::sync::Arc<[UiMountedTextForegroundSpan]>,
        ),
        UiMountedProjectionDenial,
    > {
        if source.is_empty() {
            return Ok((Box::new([]), std::sync::Arc::from([])));
        }
        match self {
            Self::Default(default) => materialize_default(default, source, constraints),
            Self::ScalarSpans(spans, _) => materialize_spans(spans, source),
        }
    }

    pub(in crate::mounting::projection) fn materialize_foregrounds(
        self,
        source: &str,
    ) -> Result<std::sync::Arc<[UiMountedTextForegroundSpan]>, UiMountedProjectionDenial> {
        if source.is_empty() {
            return Ok(std::sync::Arc::from([]));
        }
        match self {
            Self::Default(default) => {
                let end = u32::try_from(source.len())
                    .map_err(|_| UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
                let range = UiTextOriginalRange::new(0, end)
                    .ok_or(UiMountedProjectionDenial::SemanticTextShapeMismatch)?;
                Ok(std::sync::Arc::from([
                    UiMountedTextForegroundSpan::from_runtime_mounting(
                        range,
                        default.color,
                        default.paint_identity,
                    ),
                ]))
            }
            Self::ScalarSpans(spans, _) => {
                validate_scalar_span_source(spans, source)?;
                Ok(spans
                    .iter()
                    .map(|span| {
                        UiMountedTextForegroundSpan::from_runtime_mounting(
                            span.original_range,
                            span.color,
                            span.paint_identity,
                        )
                    })
                    .collect::<Vec<_>>()
                    .into())
            }
        }
    }
}

fn materialize_default(
    default: &UiMountedSemanticTextDefault,
    source: &str,
    constraints: &worth_ui_text::UiTextParagraphConstraints,
) -> Result<
    (
        Box<[worth_ui_text::UiTextStyleSpan]>,
        std::sync::Arc<[UiMountedTextForegroundSpan]>,
    ),
    UiMountedProjectionDenial,
> {
    let end = u32::try_from(source.len())
        .map_err(|_| UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
    let range = UiTextOriginalRange::from_text_mechanics(0, end)
        .ok_or(UiMountedProjectionDenial::SemanticTextShapeMismatch)?;
    let style = default
        .style
        .clone()
        .unwrap_or_else(|| worth_ui_text::UiTextStyle::from_paragraph_constraints(constraints));
    Ok((
        Box::new([worth_ui_text::UiTextStyleSpan::new(range, style)
            .ok_or(UiMountedProjectionDenial::SemanticTextShapeMismatch)?]),
        std::sync::Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
            range,
            default.color,
            default.paint_identity,
        )]),
    ))
}

fn materialize_spans(
    spans: &[UiMountedSemanticTextResolvedSpan],
    source: &str,
) -> Result<
    (
        Box<[worth_ui_text::UiTextStyleSpan]>,
        std::sync::Arc<[UiMountedTextForegroundSpan]>,
    ),
    UiMountedProjectionDenial,
> {
    validate_scalar_span_source(spans, source)?;
    let styles = spans
        .iter()
        .map(|span| {
            worth_ui_text::UiTextStyleSpan::new(span.original_range, span.style.clone())
                .ok_or(UiMountedProjectionDenial::SemanticTextShapeMismatch)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let foregrounds = spans
        .iter()
        .map(|span| {
            UiMountedTextForegroundSpan::from_runtime_mounting(
                span.original_range,
                span.color,
                span.paint_identity,
            )
        })
        .collect::<Vec<_>>();
    Ok((styles.into_boxed_slice(), foregrounds.into()))
}

fn validate_scalar_span_source(
    spans: &[UiMountedSemanticTextResolvedSpan],
    source: &str,
) -> Result<(), UiMountedProjectionDenial> {
    let exact_end = u32::try_from(source.len())
        .map_err(|_| UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
    if spans.last().map(|span| span.original_range.end()) != Some(exact_end)
        || spans.iter().any(|span| {
            !source.is_char_boundary(span.original_range.start() as usize)
                || !source.is_char_boundary(span.original_range.end() as usize)
        })
    {
        Err(UiMountedProjectionDenial::SemanticTextShapeMismatch)
    } else {
        Ok(())
    }
}
