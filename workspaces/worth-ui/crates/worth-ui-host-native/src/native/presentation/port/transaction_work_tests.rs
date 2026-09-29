//! Vertex encode reports one glyph to the presentation work ledger per glyph
//! operation it encodes, and charges nothing to any other stage.

use worth_ui_host_contract::{take_presentation_work, UiMountedRgba8, UiPresentationWorkStage};

use super::glyph_vertices;
use crate::native::presentation::pipeline_glyph_tests::{alpha_key, command};
use crate::native::presentation::UiNativeRasterOperation;
use crate::native::text_atlas::UiNativeGpuAtlasKind;

#[test]
fn vertex_encode_counts_each_glyph_operation_it_encodes() {
    let glyph = |x: f32| {
        UiNativeRasterOperation::Glyph(command(
            alpha_key(),
            UiNativeGpuAtlasKind::Alpha,
            [x, 0.0, 4.0, 8.0],
            UiMountedRgba8::new(255, 0, 0, 255),
        ))
    };
    let operations = [glyph(0.0), glyph(4.0), glyph(8.0)];
    take_presentation_work();
    let vertices = glyph_vertices(&operations, [16, 8]);
    let work = take_presentation_work();
    assert_eq!(vertices.len(), 3 * 6);
    assert_eq!(work.glyphs(UiPresentationWorkStage::VertexEncode), 3);
    assert_eq!(work.glyphs_total(), 3);

    glyph_vertices(&[], [16, 8]);
    assert_eq!(take_presentation_work().glyphs_total(), 0);
}
