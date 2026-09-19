use std::sync::{Arc, OnceLock};

pub(crate) struct UiQualifiedTextTestFixture {
    fonts: Arc<worth_ui_text::UiGlobalFontCollection>,
}

pub(crate) fn inert_qualified_layout(source: &str) -> Arc<worth_ui_text::UiQualifiedTextLayout> {
    UiQualifiedTextTestFixture::new().layout(source)
}

impl UiQualifiedTextTestFixture {
    pub(crate) fn new() -> Self {
        static QUALIFIED_FONTS: OnceLock<Arc<worth_ui_text::UiGlobalFontCollection>> =
            OnceLock::new();
        let fonts = QUALIFIED_FONTS.get_or_init(|| {
            let (fonts, _) =
                worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
            Arc::new(fonts)
        });
        Self {
            fonts: Arc::clone(fonts),
        }
    }

    pub(crate) fn layout(&self, source: &str) -> Arc<worth_ui_text::UiQualifiedTextLayout> {
        let ranges = if source.is_empty() {
            Vec::new()
        } else {
            vec![worth_ui_host_contract::UiTextOriginalRange::new(
                0,
                u32::try_from(source.len()).unwrap(),
            )
            .unwrap()]
        };
        self.layout_with_ranges(source, &ranges)
    }

    pub(crate) fn layout_with_ranges(
        &self,
        source: &str,
        ranges: &[worth_ui_host_contract::UiTextOriginalRange],
    ) -> Arc<worth_ui_text::UiQualifiedTextLayout> {
        let source: Arc<str> = Arc::from(source);
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
        let styles = ranges
            .iter()
            .map(|&range| {
                worth_ui_text::UiTextStyleSpan::new(
                    range,
                    worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
                )
                .unwrap()
            })
            .collect();
        Arc::new(
            worth_ui_text::qualify_text_layout(
                worth_ui_text::UiTextParagraphAdmissionInput {
                    source,
                    constraints,
                    profile_generation: worth_ui_host_contract::UiTextProfileGeneration::new(1)
                        .unwrap(),
                    font_collection_generation: self.fonts.generation(),
                    text_scale_generation: worth_ui_host_contract::UiTextScaleGeneration::new(1)
                        .unwrap(),
                    styles,
                },
                Arc::clone(&self.fonts),
            )
            .unwrap(),
        )
    }
}
