//! Exact semantic candidate correspondence shared by ordinary and staged text.

use worth_ui_host_contract::{UiGlyphRunView, UiMountedSemanticTextMechanic};

pub(crate) fn mechanic_contains_run(
    mechanic: &UiMountedSemanticTextMechanic,
    layout: worth_ui_host_contract::UiQualifiedTextLayoutView<'_>,
    run: UiGlyphRunView,
) -> bool {
    mechanic.qualified_layout_identity() == run.layout_identity()
        && layout.identity() == run.layout_identity()
        && run.clip_bounds() == mechanic.clip_bounds()
        && run.layer_semantic_order() == mechanic.layer_semantic_order()
        && positioned_glyph_matches(mechanic, layout, run)
        && mechanic.foregrounds().iter().any(|foreground| {
            foreground.identity() == run.paint_span()
                && range_contains(foreground.original_range(), run.original_range())
                && foreground.color() == run.foreground()
        })
}

fn positioned_glyph_matches(
    mechanic: &UiMountedSemanticTextMechanic,
    layout: worth_ui_host_contract::UiQualifiedTextLayoutView<'_>,
    run: UiGlyphRunView,
) -> bool {
    layout.positioned_glyphs().iter().any(|positioned| {
        let Some(glyph) = usize::try_from(positioned.source_glyph_index())
            .ok()
            .and_then(|index| layout.glyphs().get(index))
        else {
            return false;
        };
        glyph.glyph_id() == run.raster_key().glyph_id()
            && glyph.original_range() == run.original_range()
            && positioned.line_index() == run.line_index()
            && positioned.visual_run_index() == run.visual_run_index()
            && mounted_origin_millipoints(mechanic.origin_x(), positioned.origin_x_millipoints())
                == Some(run.origin_x_millipoints())
            && mounted_origin_millipoints(mechanic.origin_y(), positioned.origin_y_millipoints())
                == Some(run.origin_y_millipoints())
    })
}

fn mounted_origin_millipoints(origin: f32, positioned: i64) -> Option<i64> {
    if !origin.is_finite() {
        return None;
    }
    let mounted = (f64::from(origin) * 1_000.0).round();
    if mounted < i64::MIN as f64 || mounted > i64::MAX as f64 {
        return None;
    }
    (mounted as i64).checked_add(positioned)
}

fn range_contains(
    span: worth_ui_host_contract::UiTextOriginalRange,
    glyph: worth_ui_host_contract::UiTextOriginalRange,
) -> bool {
    span.start() <= glyph.start() && span.end() >= glyph.end()
}

#[cfg(test)]
mod tests {
    use super::range_contains;
    use worth_ui_host_contract::UiTextOriginalRange;
    #[test]
    fn a_glyph_cluster_must_be_contained_by_its_exact_paint_span() {
        let span = UiTextOriginalRange::new(4, 12).unwrap();
        assert!(range_contains(
            span,
            UiTextOriginalRange::new(6, 8).unwrap()
        ));
        assert!(!range_contains(
            span,
            UiTextOriginalRange::new(2, 8).unwrap()
        ));
        assert!(!range_contains(
            span,
            UiTextOriginalRange::new(8, 14).unwrap()
        ));
    }
}
