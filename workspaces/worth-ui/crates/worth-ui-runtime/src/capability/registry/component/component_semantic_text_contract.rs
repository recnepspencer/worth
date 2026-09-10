use crate::capability::ThemeTokenId;

use super::component_semantic_text_span_contract::{
    whole_paragraph_paint_identity, ComponentSemanticTextSpanContract,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentSemanticTextContractDenial {
    EmptySpans,
    SpanCapacityExceeded,
    NonContiguousSpans,
    EmptyLineHeight,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentSemanticTextContract {
    theme_token: ThemeTokenId,
    layer_semantic_order: u32,
    style: Option<worth_ui_text::UiTextStyle>,
    line_height_millipoints: Option<u32>,
    scalar_spans: Box<[ComponentSemanticTextSpanContract]>,
    default_paint_identity: [u8; 32],
}

impl ComponentSemanticTextContract {
    pub fn body_default(theme_token: ThemeTokenId, layer_semantic_order: u32) -> Self {
        let default_paint_identity = whole_paragraph_paint_identity(&theme_token);
        Self {
            theme_token,
            layer_semantic_order,
            style: None,
            line_height_millipoints: None,
            scalar_spans: Box::new([]),
            default_paint_identity,
        }
    }

    pub fn qualified(
        theme_token: ThemeTokenId,
        layer_semantic_order: u32,
        style: worth_ui_text::UiTextStyle,
    ) -> Self {
        let default_paint_identity = whole_paragraph_paint_identity(&theme_token);
        Self {
            theme_token,
            layer_semantic_order,
            style: Some(style),
            line_height_millipoints: None,
            scalar_spans: Box::new([]),
            default_paint_identity,
        }
    }

    pub fn qualified_with_line_height(
        theme_token: ThemeTokenId,
        layer_semantic_order: u32,
        style: worth_ui_text::UiTextStyle,
        line_height_millipoints: u32,
    ) -> Result<Self, ComponentSemanticTextContractDenial> {
        if line_height_millipoints == 0 {
            return Err(ComponentSemanticTextContractDenial::EmptyLineHeight);
        }
        let default_paint_identity = whole_paragraph_paint_identity(&theme_token);
        Ok(Self {
            theme_token,
            layer_semantic_order,
            style: Some(style),
            line_height_millipoints: Some(line_height_millipoints),
            scalar_spans: Box::new([]),
            default_paint_identity,
        })
    }

    pub fn spanned(
        theme_token: ThemeTokenId,
        layer_semantic_order: u32,
        scalar_spans: impl IntoIterator<Item = ComponentSemanticTextSpanContract>,
    ) -> Result<Self, ComponentSemanticTextContractDenial> {
        let scalar_spans = scalar_spans.into_iter().collect::<Vec<_>>();
        validate_spans(&scalar_spans)?;
        let default_paint_identity = whole_paragraph_paint_identity(&theme_token);
        Ok(Self {
            theme_token,
            layer_semantic_order,
            style: None,
            line_height_millipoints: None,
            scalar_spans: scalar_spans.into_boxed_slice(),
            default_paint_identity,
        })
    }

    pub fn with_scalar_spans(
        mut self,
        scalar_spans: impl IntoIterator<Item = ComponentSemanticTextSpanContract>,
    ) -> Result<Self, ComponentSemanticTextContractDenial> {
        let scalar_spans = scalar_spans.into_iter().collect::<Vec<_>>();
        validate_spans(&scalar_spans)?;
        self.scalar_spans = scalar_spans.into_boxed_slice();
        Ok(self)
    }

    pub fn theme_token(&self) -> &ThemeTokenId {
        &self.theme_token
    }

    pub fn layer_semantic_order(&self) -> u32 {
        self.layer_semantic_order
    }

    pub fn style(&self) -> Option<&worth_ui_text::UiTextStyle> {
        self.style.as_ref()
    }

    pub const fn line_height_millipoints(&self) -> Option<u32> {
        self.line_height_millipoints
    }

    pub fn scalar_spans(&self) -> &[ComponentSemanticTextSpanContract] {
        &self.scalar_spans
    }

    pub fn foreground_tokens(&self) -> impl Iterator<Item = &ThemeTokenId> {
        std::iter::once(&self.theme_token)
            .chain(self.scalar_spans.iter().map(|span| span.foreground_token()))
    }

    pub(crate) const fn default_paint_identity(&self) -> [u8; 32] {
        self.default_paint_identity
    }

    pub(crate) fn digest_basis(&self) -> String {
        let style = self.style.as_ref().map_or_else(
            || "body-default".to_owned(),
            |style| {
                style
                    .identity_digest()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect()
            },
        );
        let spans = self
            .scalar_spans
            .iter()
            .fold(String::new(), |mut digest, span| {
                use std::fmt::Write;
                let _ = write!(
                    digest,
                    ":{}-{}:{}:appearance:{}:",
                    span.original_range().start(),
                    span.original_range().end(),
                    span.foreground_token().as_str(),
                    span.uses_appearance_foreground()
                );
                for byte in span.style().identity_digest() {
                    let _ = write!(digest, "{byte:02x}");
                }
                digest
            });
        format!(
            "semantic-text:{}:{}:{style}:line-height:{:?}",
            self.theme_token.as_str(),
            self.layer_semantic_order,
            self.line_height_millipoints,
        ) + &spans
    }
}

fn validate_spans(
    spans: &[ComponentSemanticTextSpanContract],
) -> Result<(), ComponentSemanticTextContractDenial> {
    if spans.is_empty() {
        return Err(ComponentSemanticTextContractDenial::EmptySpans);
    }
    if spans.len() > worth_ui_text::UiGlobalTextProfile::MAX_RUNS_PER_PARAGRAPH {
        return Err(ComponentSemanticTextContractDenial::SpanCapacityExceeded);
    }
    let contiguous = spans
        .first()
        .is_some_and(|span| span.original_range().start() == 0)
        && spans
            .windows(2)
            .all(|pair| pair[0].original_range().end() == pair[1].original_range().start());
    if !contiguous {
        return Err(ComponentSemanticTextContractDenial::NonContiguousSpans);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn authored_spans_must_be_nonempty_contiguous_and_bounded() {
        let token = ThemeTokenId::new("theme.text").unwrap();
        let first = span(0, 2, &token);
        let gap = span(3, 4, &token);
        assert_eq!(
            ComponentSemanticTextContract::spanned(token.clone(), 1, []),
            Err(ComponentSemanticTextContractDenial::EmptySpans)
        );
        assert_eq!(
            ComponentSemanticTextContract::spanned(token.clone(), 1, [first.clone(), gap]),
            Err(ComponentSemanticTextContractDenial::NonContiguousSpans)
        );
        let too_many = (0..=worth_ui_text::UiGlobalTextProfile::MAX_RUNS_PER_PARAGRAPH)
            .map(|index| span(index as u32, index as u32 + 1, &token));
        assert_eq!(
            ComponentSemanticTextContract::spanned(token.clone(), 1, too_many),
            Err(ComponentSemanticTextContractDenial::SpanCapacityExceeded)
        );
    }

    #[test]
    fn qualified_line_height_is_explicit_and_nonzero() {
        let token = ThemeTokenId::new("theme.text").unwrap();
        let constraints = worth_ui_text::UiTextParagraphConstraints::new(
            worth_ui_text::UiTextParagraphConstraintsInput {
                language: std::sync::Arc::from("und"),
                base_direction: worth_ui_text::UiTextBaseDirection::Auto,
                wrap: worth_ui_text::UiTextWrap::UnicodeWord,
                alignment: worth_ui_text::UiTextAlignment::Start,
                overflow: worth_ui_text::UiTextOverflow::Clip,
                font_size_millipoints: 28_000,
                width_millipoints: 320_000,
                line_height_millipoints: 36_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                tab_interval_millipoints: 112_000,
                maximum_lines: 1,
            },
        )
        .unwrap();
        let style = worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints);
        assert_eq!(
            ComponentSemanticTextContract::qualified_with_line_height(
                token.clone(),
                1,
                style.clone(),
                0,
            ),
            Err(ComponentSemanticTextContractDenial::EmptyLineHeight),
        );
        let contract =
            ComponentSemanticTextContract::qualified_with_line_height(token, 1, style, 36_000)
                .unwrap();
        assert_eq!(contract.line_height_millipoints(), Some(36_000));
    }

    #[test]
    fn qualified_contract_can_adopt_canonical_spans_without_losing_typography() {
        let token = ThemeTokenId::new("theme.text").unwrap();
        let constraints = worth_ui_text::UiTextParagraphConstraints::new(
            worth_ui_text::UiTextParagraphConstraintsInput {
                language: std::sync::Arc::from("und"),
                base_direction: worth_ui_text::UiTextBaseDirection::Auto,
                wrap: worth_ui_text::UiTextWrap::UnicodeWord,
                alignment: worth_ui_text::UiTextAlignment::Start,
                overflow: worth_ui_text::UiTextOverflow::Clip,
                font_size_millipoints: 28_000,
                width_millipoints: 320_000,
                line_height_millipoints: 36_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                tab_interval_millipoints: 112_000,
                maximum_lines: 1,
            },
        )
        .unwrap();
        let style = worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints);
        let adopted = ComponentSemanticTextContract::qualified_with_line_height(
            token.clone(),
            1,
            style.clone(),
            36_000,
        )
        .unwrap()
        .with_scalar_spans([span(0, 4, &token).with_appearance_foreground()])
        .unwrap();

        assert_eq!(adopted.style(), Some(&style));
        assert_eq!(adopted.line_height_millipoints(), Some(36_000));
        assert!(adopted.scalar_spans()[0].uses_appearance_foreground());
    }

    fn span(start: u32, end: u32, token: &ThemeTokenId) -> ComponentSemanticTextSpanContract {
        let constraints = worth_ui_text::UiTextParagraphConstraints::new(
            worth_ui_text::UiTextParagraphConstraintsInput {
                language: Arc::from("und"),
                base_direction: worth_ui_text::UiTextBaseDirection::Auto,
                wrap: worth_ui_text::UiTextWrap::UnicodeWord,
                alignment: worth_ui_text::UiTextAlignment::Start,
                overflow: worth_ui_text::UiTextOverflow::Clip,
                font_size_millipoints: 14_000,
                width_millipoints: 160_000,
                line_height_millipoints: 18_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                tab_interval_millipoints: 56_000,
                maximum_lines: 1,
            },
        )
        .unwrap();
        ComponentSemanticTextSpanContract::new(
            worth_ui_host_contract::UiTextOriginalRange::new(start, end).unwrap(),
            token.clone(),
            worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
        )
        .unwrap()
    }
}
