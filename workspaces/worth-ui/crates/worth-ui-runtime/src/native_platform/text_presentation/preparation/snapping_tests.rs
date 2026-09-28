//! A text command that layout moves by a fraction of a device pixel draws its
//! glyphs from the same device pixel, so a resize that nudges it asks the atlas
//! for nothing new.

use super::tests::mixed_bidi_paint_world;
use super::{prepare_demands, MountedTextDemandJoin, UiMountedEventTimeDpiAuthority};
use std::num::NonZeroU32;
use worth_ui_host_contract::UiGlyphRasterKey;

struct Drawn {
    keys: Vec<UiGlyphRasterKey>,
    run_origins: Vec<[i64; 2]>,
}

fn drawn_at(origin: [f32; 2], dpi_milli: u32) -> Drawn {
    let (layout, mechanic, command, damage) = mixed_bidi_paint_world(origin);
    let join = MountedTextDemandJoin {
        dpi: UiMountedEventTimeDpiAuthority(NonZeroU32::new(dpi_milli).unwrap()),
        lane: worth_ui_host_contract::UiGlyphRasterLane::Ordinary,
        selection: worth_ui_text::UiGlyphRasterDemandSelection::LogicalDamage(&damage),
        resolve: |_| Some(layout.as_ref()),
        _layout: std::marker::PhantomData,
    };
    let prepared = prepare_demands(&[(command, &mechanic)], &join).unwrap();
    let keys = prepared
        .demands
        .iter()
        .flat_map(|batch| batch.records().iter().map(|record| record.key()))
        .collect::<Vec<_>>();
    assert!(!keys.is_empty());
    let run_origins = prepared
        .glyph_runs
        .iter()
        .map(|run| [run.origin_x_millipoints(), run.origin_y_millipoints()])
        .collect();
    Drawn { keys, run_origins }
}

#[test]
fn a_move_within_one_device_pixel_keeps_every_key_and_run_origin() {
    // 1.5 device pixels per point: 10.1 and 10.2 points both draw from
    // device pixel 15.
    let before = drawn_at([10.1, 20.1], 1_500);
    let after = drawn_at([10.2, 20.2], 1_500);
    assert_eq!(before.keys, after.keys);
    assert_eq!(before.run_origins, after.run_origins);
}

#[test]
fn a_move_to_another_device_pixel_moves_the_runs_by_whole_pixels_and_keeps_the_keys() {
    // 10.1 points draws from pixel 15, 10.4 points from pixel 16: two thirds
    // of a point further right at 1.5 pixels per point.
    let before = drawn_at([10.1, 20.1], 1_500);
    let after = drawn_at([10.4, 20.1], 1_500);
    assert_eq!(before.keys, after.keys);
    assert_eq!(before.run_origins.len(), after.run_origins.len());
    for (before, after) in before.run_origins.iter().zip(&after.run_origins) {
        assert_eq!(after[0] - before[0], 667);
        assert_eq!(after[1], before[1]);
    }
}

#[test]
fn the_same_move_at_a_scale_where_it_crosses_a_pixel_moves_the_runs() {
    // At one pixel per point 10.4 points still draws from pixel 10, while at
    // two pixels per point 10.1 and 10.4 points draw from pixels 20 and 21.
    assert_eq!(
        drawn_at([10.1, 20.0], 1_000).run_origins,
        drawn_at([10.4, 20.0], 1_000).run_origins
    );
    assert_ne!(
        drawn_at([10.1, 20.0], 2_000).run_origins,
        drawn_at([10.4, 20.0], 2_000).run_origins
    );
}
