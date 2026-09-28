use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Finds a layout an earlier frame already shaped for the request with a
/// reflow key, at a width in millipoints.
pub(in crate::mounting::projection) type UiMountedRetainedTextLayouts = Box<
    dyn Fn(
        worth_ui_text::UiQualifiedTextReflowKey,
        u32,
    ) -> Option<Arc<worth_ui_text::UiQualifiedTextLayout>>,
>;

/// Shapes each distinct layout once. A request that differs from the one a
/// layout was shaped for only in a width that layout admits reuses it, so
/// text that only moves, whose box changes height, or whose width changes
/// without refitting its lines keeps its shaping.
///
/// Layouts are ordered by reflow key and least width. The layouts of one
/// request admit disjoint widths, so the only one that can admit a width is
/// the nearest at or below it.
pub(in crate::mounting::projection) struct UiMountedTextQualificationCache {
    retained: UiMountedRetainedTextLayouts,
    layouts: RefCell<BTreeMap<([u8; 32], u32), Arc<worth_ui_text::UiQualifiedTextLayout>>>,
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
        let reflow = request.reflow_key();
        let width = request.width_millipoints();
        if let Some((_, layout)) = self
            .layouts
            .borrow()
            .range(..=(reflow.digest(), width))
            .next_back()
            .filter(|((digest, _), layout)| {
                *digest == reflow.digest() && layout.admits_width(width)
            })
        {
            return Ok((Arc::clone(layout), false));
        }
        let reused = (self.retained)(reflow, width);
        let shaped = reused.is_none();
        let layout = match reused {
            Some(layout) => layout,
            None => Arc::new(request.qualify()?),
        };
        self.layouts.borrow_mut().insert(
            (reflow.digest(), layout.least_width_millipoints()),
            Arc::clone(&layout),
        );
        Ok((layout, shaped))
    }
}

/// A single-style request for `source`, for tests of layout reuse.
#[cfg(test)]
pub(in crate::mounting::projection) fn request_for_test(
    source: &str,
    width_millipoints: u32,
    maximum_lines: u32,
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
            width_millipoints,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines,
        },
    )
    .unwrap();
    let range =
        worth_ui_host_contract::UiTextOriginalRange::new(0, u32::try_from(source.len()).unwrap())
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
            profile_generation: worth_ui_host_contract::UiTextProfileGeneration::new(1).unwrap(),
            font_collection_generation: fonts.generation(),
            text_scale_generation: worth_ui_host_contract::UiTextScaleGeneration::new(1).unwrap(),
            styles: Box::new([style]),
        },
        fonts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_requests_share_layouts_without_collapsing_distinct_text() {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let fonts = Arc::new(fonts);
        let cache = UiMountedTextQualificationCache::reusing(Box::new(|_, _| None));
        let repeated_request = request_for_test("same", 160_000, 1, Arc::clone(&fonts));

        let (first, first_shaped) = cache.qualify(repeated_request.clone()).unwrap();
        let (repeated, repeated_shaped) = cache.qualify(repeated_request).unwrap();
        let (distinct, distinct_shaped) = cache
            .qualify(request_for_test(
                "different",
                160_000,
                1,
                Arc::clone(&fonts),
            ))
            .unwrap();

        assert!(Arc::ptr_eq(&first, &repeated));
        assert!(!Arc::ptr_eq(&first, &distinct));
        assert_eq!(
            (first_shaped, repeated_shaped, distinct_shaped),
            (true, false, true)
        );
    }

    /// One request held at two widths that fit its lines differently: each
    /// width finds the layout that admits it, not merely one for its request.
    #[test]
    fn each_width_finds_the_layout_that_admits_it() {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let fonts = Arc::new(fonts);
        let cache = UiMountedTextQualificationCache::reusing(Box::new(|_, _| None));
        let label = |width| request_for_test("office hours and more", width, 8, Arc::clone(&fonts));

        let (wide, _) = cache.qualify(label(400_000)).unwrap();
        let (narrow, narrow_shaped) = cache.qualify(label(40_000)).unwrap();
        assert!(narrow_shaped);
        assert!(narrow.lines().len() > wide.lines().len());

        let (wider, wider_shaped) = cache.qualify(label(600_000)).unwrap();
        let (again, again_shaped) = cache.qualify(label(40_000)).unwrap();
        assert!(Arc::ptr_eq(&wider, &wide) && !wider_shaped);
        assert!(Arc::ptr_eq(&again, &narrow) && !again_shaped);
    }

    /// A width between two held layouts of one request is admitted by
    /// neither, so it shapes its own rather than taking the nearest below it.
    #[test]
    fn a_width_between_held_layouts_shapes_its_own() {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let fonts = Arc::new(fonts);
        let cache = UiMountedTextQualificationCache::reusing(Box::new(|_, _| None));
        let label = |width| request_for_test("office hours and more", width, 8, Arc::clone(&fonts));

        let (narrow, _) = cache.qualify(label(40_000)).unwrap();
        let (wide, _) = cache.qualify(label(400_000)).unwrap();
        let between = (40_000..400_000)
            .step_by(1_000)
            .find(|width| !narrow.admits_width(*width) && !wide.admits_width(*width))
            .unwrap();
        let (layout, shaped) = cache.qualify(label(between)).unwrap();

        assert!(shaped);
        assert!(!Arc::ptr_eq(&layout, &narrow) && !Arc::ptr_eq(&layout, &wide));
    }

    #[test]
    fn a_retained_layout_for_the_same_request_is_reused_unshaped() {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let fonts = Arc::new(fonts);
        let retained_request = request_for_test("retained", 160_000, 1, Arc::clone(&fonts));
        let retained_reflow = retained_request.reflow_key();
        let retained = Arc::new(retained_request.clone().qualify().unwrap());
        let held = Arc::clone(&retained);
        let cache = UiMountedTextQualificationCache::reusing(Box::new(move |reflow, width| {
            (reflow == retained_reflow && held.admits_width(width)).then(|| Arc::clone(&held))
        }));

        let (reused, reused_shaped) = cache.qualify(retained_request).unwrap();
        let (fresh, fresh_shaped) = cache
            .qualify(request_for_test("fresh", 160_000, 1, Arc::clone(&fonts)))
            .unwrap();

        assert!(Arc::ptr_eq(&reused, &retained));
        assert!(!reused_shaped);
        assert!(fresh_shaped);
        assert!(!Arc::ptr_eq(&fresh, &retained));
    }
}
