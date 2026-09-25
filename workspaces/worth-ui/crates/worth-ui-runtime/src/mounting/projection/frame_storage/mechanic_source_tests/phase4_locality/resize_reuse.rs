//! Resizing reshapes text only where its shaping inputs change: the allocated
//! width and the declared flow. Moving a paragraph, or changing only its box
//! height, reuses the layout an earlier frame shaped.

use super::*;
use crate::capability::ComponentSemanticTextFlow;
use worth_ui_host_contract::UiSemanticTextSlot;

const HOME: [f32; 4] = [0.0, 0.0, 160.0, 96.0];

#[test]
fn moving_a_paragraph_keeps_its_shaping() {
    let mut world = ResizeWorld::mounted(UiMountedSemanticTextSeed::scalar_for_test(), HOME);
    let before = world.layouts();

    world.resize([40.0, 24.0, 160.0, 96.0]);

    world.assert_reused(&before);
}

#[test]
fn a_height_only_change_keeps_its_shaping() {
    let mut world = ResizeWorld::mounted(UiMountedSemanticTextSeed::scalar_for_test(), HOME);
    let before = world.layouts();

    world.resize([0.0, 0.0, 160.0, 18.0]);
    world.assert_reused(&before);
    world.resize([0.0, 0.0, 160.0, 400.0]);
    world.assert_reused(&before);
}

#[test]
fn a_width_change_reshapes_and_the_new_width_is_then_reused() {
    let mut world = ResizeWorld::mounted(UiMountedSemanticTextSeed::scalar_for_test(), HOME);
    let before = world.layouts();

    world.resize([0.0, 0.0, 80.0, 96.0]);
    let narrow = world.layouts();
    world.assert_reshaped(&before);

    world.resize([12.0, 0.0, 80.0, 30.0]);
    world.assert_reused(&narrow);
}

#[test]
fn a_flow_change_at_the_same_width_reshapes() {
    let label = |flow| UiMountedSemanticTextSeed::scalar_text_for_test("quarterly revenue", flow);
    let mut world = ResizeWorld::mounted(label(ComponentSemanticTextFlow::wrapping()), HOME);
    let before = world.layouts();

    world.seed = label(ComponentSemanticTextFlow::single_line_ellipsis());
    world.resize(HOME);

    world.assert_reshaped(&before);
}

#[test]
fn declared_flow_wraps_or_ellipsizes_at_the_allocated_width() {
    const LABEL: &str = "quarterly revenue by region";
    let narrow = [0.0, 0.0, 60.0, 18.0];
    let wrapping = ResizeWorld::mounted(
        UiMountedSemanticTextSeed::scalar_text_for_test(
            LABEL,
            ComponentSemanticTextFlow::wrapping(),
        ),
        narrow,
    )
    .value_layout();
    assert!(wrapping.lines().len() > 1);

    let single = UiMountedSemanticTextSeed::scalar_text_for_test(
        LABEL,
        ComponentSemanticTextFlow::single_line_ellipsis(),
    );
    let mut world = ResizeWorld::mounted(single, narrow);
    let ellipsized = world.value_layout();
    let [line] = ellipsized.lines() else {
        panic!("a single-line flow shapes one line")
    };
    assert!(line.overflowed());
    let last = ellipsized.positioned_glyphs().last().unwrap();
    assert!(ellipsized.glyphs()[last.source_glyph_index() as usize]
        .original_range()
        .is_empty());

    world.resize([0.0, 0.0, 600.0, 18.0]);
    let wide = world.value_layout();
    let [line] = wide.lines() else {
        panic!("a single-line flow shapes one line")
    };
    assert!(!line.overflowed());
}

/// Two paragraphs share a request. When both move to a new font collection
/// one after the other, the second reuses the layout the first was shaped
/// with there; the old collection's layout leaves with its last holder.
#[test]
fn a_collection_change_reuses_the_layout_shaped_for_the_new_collection() {
    let admit = || {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        Arc::new(fonts)
    };
    let (old_fonts, new_fonts) = (admit(), admit());
    let mut source = UiMountedMechanicSource::default();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let left = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let right = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let paragraph = |instance, node, generation| InstanceFixture {
        instance,
        surface,
        binding,
        node: crate::graph::UiGraphNodeIdentity::new(node),
        seed: UiMountedSemanticTextSeed::scalar_for_test(),
        extent: HOME,
        generation,
    };
    apply_instance(&mut source, &old_fonts, paragraph(left, 4_201, 1));
    apply_instance(&mut source, &old_fonts, paragraph(right, 4_202, 1));

    apply_instance(&mut source, &new_fonts, paragraph(right, 4_202, 2));
    apply_instance(&mut source, &new_fonts, paragraph(left, 4_201, 2));

    let shaped_first = layouts(&source, right, surface, binding);
    let reused = layouts(&source, left, surface, binding);
    assert!(!shaped_first.is_empty());
    assert_eq!(reused.len(), shaped_first.len());
    assert!(shaped_first
        .iter()
        .zip(&reused)
        .all(|(first, reused)| Arc::ptr_eq(first, reused)
            && Arc::ptr_eq(reused.pinned_font_collection(), &new_fonts)));
    assert!(semantic_rows(&source, left, surface, binding)
        .iter()
        .all(|row| row.performed_layout_cost().is_none()));
    let held = reused
        .iter()
        .map(|layout| Arc::as_ptr(layout) as usize)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(source.qualified_layout_count(), held.len());
}

struct ResizeWorld {
    fonts: Arc<worth_ui_text::UiGlobalFontCollection>,
    source: UiMountedMechanicSource,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    seed: UiMountedSemanticTextSeed,
    generation: u64,
}

impl ResizeWorld {
    fn mounted(seed: UiMountedSemanticTextSeed, extent: [f32; 4]) -> Self {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let mut world = Self {
            fonts: Arc::new(fonts),
            source: UiMountedMechanicSource::default(),
            instance: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            seed,
            generation: 0,
        };
        world.resize(extent);
        world
    }

    fn resize(&mut self, extent: [f32; 4]) {
        self.generation += 1;
        apply_instance(
            &mut self.source,
            &self.fonts,
            InstanceFixture {
                instance: self.instance,
                surface: self.surface,
                binding: self.binding,
                node: crate::graph::UiGraphNodeIdentity::new(4_142),
                seed: self.seed.clone(),
                extent,
                generation: self.generation,
            },
        );
    }

    fn rows(&self) -> Vec<worth_ui_host_contract::UiMountedSemanticTextMechanic> {
        semantic_rows(&self.source, self.instance, self.surface, self.binding)
    }

    fn layouts(&self) -> Vec<Arc<worth_ui_text::UiQualifiedTextLayout>> {
        layouts(&self.source, self.instance, self.surface, self.binding)
    }

    fn value_layout(&self) -> Arc<worth_ui_text::UiQualifiedTextLayout> {
        Arc::clone(
            self.source
                .qualified_layout_for(self.instance, UiSemanticTextSlot::Value)
                .unwrap(),
        )
    }

    /// Every row was shaped again into a layout it did not hold before.
    fn assert_reshaped(&self, before: &[Arc<worth_ui_text::UiQualifiedTextLayout>]) {
        let after = self.layouts();
        assert!(!before.is_empty());
        assert_eq!(after.len(), before.len());
        assert!(before
            .iter()
            .zip(&after)
            .all(|(before, after)| !Arc::ptr_eq(before, after)));
        assert!(self
            .rows()
            .iter()
            .all(|row| row.performed_layout_cost().is_some()));
    }

    /// Every row was completed again for the new box, from the layouts it
    /// held before, with no shaping performed.
    fn assert_reused(&self, before: &[Arc<worth_ui_text::UiQualifiedTextLayout>]) {
        let after = self.layouts();
        assert!(!before.is_empty());
        assert_eq!(after.len(), before.len());
        assert!(before
            .iter()
            .zip(&after)
            .all(|(before, after)| Arc::ptr_eq(before, after)));
        assert!(self
            .rows()
            .iter()
            .all(|row| row.performed_layout_cost().is_none()));
    }
}
