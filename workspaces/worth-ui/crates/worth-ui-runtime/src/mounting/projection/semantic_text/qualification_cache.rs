use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Finds a layout an earlier frame already shaped for the same request.
pub(in crate::mounting::projection) type UiMountedRetainedTextLayouts = Box<
    dyn Fn(
        worth_ui_host_contract::UiQualifiedTextLayoutRequestIdentity,
    ) -> Option<Arc<worth_ui_text::UiQualifiedTextLayout>>,
>;

/// Shapes each distinct request once. A request equal to one a retained row
/// was shaped for reuses that layout, so text that only moves, or whose box
/// changes height, keeps its shaping.
pub(in crate::mounting::projection) struct UiMountedTextQualificationCache {
    retained: UiMountedRetainedTextLayouts,
    layouts: RefCell<HashMap<[u8; 32], Arc<worth_ui_text::UiQualifiedTextLayout>>>,
}

impl UiMountedTextQualificationCache {
    pub(in crate::mounting::projection) fn reusing(retained: UiMountedRetainedTextLayouts) -> Self {
        Self {
            retained,
            layouts: RefCell::default(),
        }
    }

    /// The layout for `request`, and whether this call shaped it.
    pub(super) fn qualify(
        &self,
        request: worth_ui_text::UiQualifiedTextLayoutRequest,
    ) -> Result<
        (Arc<worth_ui_text::UiQualifiedTextLayout>, bool),
        worth_ui_text::UiTextQualificationDenial,
    > {
        let identity = request.identity();
        if let Some(layout) = self.layouts.borrow().get(&identity.digest()) {
            return Ok((Arc::clone(layout), false));
        }
        let reused = (self.retained)(identity);
        let shaped = reused.is_none();
        let layout = match reused {
            Some(layout) => layout,
            None => Arc::new(request.qualify()?),
        };
        self.layouts
            .borrow_mut()
            .insert(identity.digest(), Arc::clone(&layout));
        Ok((layout, shaped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_requests_share_layouts_without_collapsing_distinct_text() {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let fonts = Arc::new(fonts);
        let cache = UiMountedTextQualificationCache::reusing(Box::new(|_| None));
        let repeated_request = request("same", Arc::clone(&fonts));

        let (first, first_shaped) = cache.qualify(repeated_request.clone()).unwrap();
        let (repeated, repeated_shaped) = cache.qualify(repeated_request).unwrap();
        let (distinct, distinct_shaped) = cache
            .qualify(request("different", Arc::clone(&fonts)))
            .unwrap();

        assert!(Arc::ptr_eq(&first, &repeated));
        assert!(!Arc::ptr_eq(&first, &distinct));
        assert_eq!(
            (first_shaped, repeated_shaped, distinct_shaped),
            (true, false, true)
        );
    }

    #[test]
    fn a_retained_layout_for_the_same_request_is_reused_unshaped() {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let fonts = Arc::new(fonts);
        let retained_request = request("retained", Arc::clone(&fonts));
        let retained_identity = retained_request.identity();
        let retained = Arc::new(retained_request.clone().qualify().unwrap());
        let held = Arc::clone(&retained);
        let cache = UiMountedTextQualificationCache::reusing(Box::new(move |identity| {
            (identity == retained_identity).then(|| Arc::clone(&held))
        }));

        let (reused, reused_shaped) = cache.qualify(retained_request).unwrap();
        let (fresh, fresh_shaped) = cache.qualify(request("fresh", Arc::clone(&fonts))).unwrap();

        assert!(Arc::ptr_eq(&reused, &retained));
        assert!(!reused_shaped);
        assert!(fresh_shaped);
        assert!(!Arc::ptr_eq(&fresh, &retained));
    }

    fn request(
        source: &str,
        fonts: Arc<worth_ui_text::UiGlobalFontCollection>,
    ) -> worth_ui_text::UiQualifiedTextLayoutRequest {
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
        let range = worth_ui_host_contract::UiTextOriginalRange::new(
            0,
            u32::try_from(source.len()).unwrap(),
        )
        .unwrap();
        let style = worth_ui_text::UiTextStyleSpan::new(
            range,
            worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
        )
        .unwrap();
        worth_ui_text::UiQualifiedTextLayoutRequest::new(
            worth_ui_text::UiTextParagraphAdmissionInput {
                source,
                constraints,
                profile_generation: worth_ui_host_contract::UiTextProfileGeneration::new(1)
                    .unwrap(),
                font_collection_generation: fonts.generation(),
                text_scale_generation: worth_ui_host_contract::UiTextScaleGeneration::new(1)
                    .unwrap(),
                styles: Box::new([style]),
            },
            fonts,
        )
    }
}
