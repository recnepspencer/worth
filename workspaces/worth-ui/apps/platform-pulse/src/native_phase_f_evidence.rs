pub(crate) fn intrinsic_glyphs(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> Vec<serde_json::Value> {
    receipt
        .final_frame()
        .intrinsic_glyphs()
        .iter()
        .copied()
        .map(|glyph| {
            serde_json::json!({
                "glyph_id": glyph.glyph_id(),
                "palette": glyph.palette(),
                "source": format!("{:?}", glyph.source()),
                "raster_key": hex_digest(glyph.raster_key_digest()),
                "original_range": glyph.original_range(),
                "foreground": glyph.foreground_rgba8(),
                "target_bounds": glyph.target_bounds(),
                "transcript_digest": hex_digest(glyph.transcript_digest()),
            })
        })
        .collect()
}

pub(crate) fn alpha_glyphs(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> Vec<serde_json::Value> {
    receipt
        .final_frame()
        .alpha_glyphs()
        .iter()
        .copied()
        .map(|glyph| {
            serde_json::json!({
                "glyph_id": glyph.glyph_id(),
                "source": format!("{:?}", glyph.source()),
                "raster_key": hex_digest(glyph.raster_key_digest()),
                "original_range": glyph.original_range(),
                "foreground": glyph.foreground_rgba8(),
                "target_bounds": glyph.target_bounds(),
                "transcript_digest": hex_digest(glyph.transcript_digest()),
            })
        })
        .collect()
}

pub(crate) fn pin_frames(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> Vec<serde_json::Value> {
    receipt
        .text_pin_frame_observations()
        .iter()
        .map(|frame| {
            frame
                .iter()
                .copied()
                .map(|pin| {
                    serde_json::json!({
                        "layout": hex_digest(pin.layout_digest()),
                        "raster_key": hex_digest(pin.raster_key_digest()),
                    })
                })
                .collect::<Vec<_>>()
        })
        .map(serde_json::Value::Array)
        .collect()
}

pub(crate) fn retained_frame_intrinsic_glyphs(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> Vec<serde_json::Value> {
    receipt
        .retained_frames()
        .iter()
        .map(|frame| {
            serde_json::json!({
                "frame": frame.frame(),
                "glyphs": frame.intrinsic_glyphs().iter().copied().map(|glyph| {
                    serde_json::json!({
                        "source": format!("{:?}", glyph.source()),
                        "raster_key": hex_digest(glyph.raster_key_digest()),
                        "original_range": glyph.original_range(),
                        "foreground": glyph.foreground_rgba8(),
                        "target_bounds": glyph.target_bounds(),
                        "transcript_digest": hex_digest(glyph.transcript_digest()),
                    })
                }).collect::<Vec<_>>(),
                "intrinsic_transcript_digest": hex_digest(frame.intrinsic_glyph_transcript_digest()),
            })
        })
        .collect()
}

pub(crate) fn hex_digest(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
