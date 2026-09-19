use super::{
    capture, incidental_visual, NativeBoundExecutableWorld, PlatformPulsePortalJourneyFailure,
    PIXEL_POLL_SLICE, TRANSITION_DEADLINE,
};
use std::time::Instant;
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseThemeSettlementPosture,
};

pub(super) fn exercise(
    world: &mut NativeBoundExecutableWorld,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    let mut previous_binding = None;
    for (revision, theme, canvas, accent, action) in [
        (1, "alternate", [24, 18, 13], [242, 173, 103], [160, 84, 24]),
        (2, "default", [11, 15, 20], [172, 103, 242], [148, 64, 212]),
    ] {
        let path = world
            .installation
            .source_root()
            .join("platform-pulse-theme.json");
        std::fs::write(
            &path,
            format!(r#"{{"schema_version":1,"revision":{revision},"theme":"{theme}"}}"#),
        )
        .map_err(|error| {
            PlatformPulsePortalJourneyFailure::UnexpectedObservation(format!(
                "theme input write: {error}"
            ))
        })?;
        let deadline = Instant::now() + TRANSITION_DEADLINE;
        loop {
            let envelope = world.lifecycle.next(deadline).map_err(|error| {
                PlatformPulsePortalJourneyFailure::UnexpectedObservation(format!(
                    "theme settlement: {error:?}"
                ))
            })?;
            match envelope.outcome() {
                PlatformPulseLifecycleObservation::ThemeSwitchSettled(theme_result) => {
                    if theme_result.preference_revision != revision
                        || theme_result.definition != format!("theme.platform_pulse.{theme}")
                        || theme_result.posture != PlatformPulseThemeSettlementPosture::Published
                        || theme_result.selected_instances == 0
                        || theme_result.materialized_contexts == 0
                        || previous_binding
                            .is_some_and(|previous| theme_result.binding_generation != previous + 1)
                    {
                        return Err(PlatformPulsePortalJourneyFailure::UnexpectedObservation(
                            format!("incorrect theme settlement: {theme_result:?}"),
                        ));
                    }
                    previous_binding = Some(theme_result.binding_generation);
                    break;
                }
                outcome if incidental_visual(outcome) => {}
                outcome => return Err(super::unexpected(outcome)),
            }
        }
        // Expected RGB values are stated independently of the application palette.
        // The canvas sample lies in the empty outer gutter. The brand region must
        // contain fully covered glyph pixels, proving text foreground also switched.
        loop {
            let pixels = capture(world)?;
            let sample_x = pixels.width() * 8 / 960;
            let sample_y = pixels.height() * 8 / 600;
            let sample = rgb(&pixels, sample_x, sample_y);
            let action_sample = rgb(
                &pixels,
                pixels.width() * 304 / 960,
                pixels.height() * 424 / 600,
            );
            let mut glyphs = 0;
            for y in pixels.height() * 24 / 600..pixels.height() * 88 / 600 {
                for x in pixels.width() * 24 / 960..pixels.width() * 240 / 960 {
                    if matches(rgb(&pixels, x, y), accent) {
                        glyphs += 1;
                    }
                }
            }
            if matches(sample, canvas)
                && matches(action_sample, action)
                && glyphs >= 8
                && action_text_visible(&pixels)
            {
                break;
            }
            if Instant::now() >= deadline {
                return Err(PlatformPulsePortalJourneyFailure::UnexpectedObservation(format!("theme {theme} pixels: canvas={sample:?}, expected={canvas:?}, accent glyphs={glyphs}, action={action_sample:?}, expected action={action:?}")));
            }
            std::thread::sleep(PIXEL_POLL_SLICE);
        }
    }
    Ok(())
}

fn action_text_visible(pixels: &crate::external_observation::NativeClientPixelCapture) -> bool {
    let mut glyphs = 0;
    for y in pixels.height() * 428 / 600..pixels.height() * 452 / 600 {
        for x in pixels.width() * 320 / 960..pixels.width() * 488 / 960 {
            if matches(rgb(pixels, x, y), [242, 244, 247]) {
                glyphs += 1;
            }
        }
    }
    glyphs >= 8
}

fn rgb(pixels: &crate::external_observation::NativeClientPixelCapture, x: u32, y: u32) -> [u8; 3] {
    let offset = ((y * pixels.width() + x) * 4) as usize;
    pixels.rgba()[offset..offset + 3].try_into().unwrap()
}

fn matches(actual: [u8; 3], expected: [u8; 3]) -> bool {
    actual
        .into_iter()
        .zip(expected)
        .all(|(a, b)| a.abs_diff(b) <= 3)
}
