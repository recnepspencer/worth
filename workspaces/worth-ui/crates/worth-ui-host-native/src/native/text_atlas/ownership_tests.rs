use super::ownership::UiNativeTextAtlas;
use crate::native::text_atlas::settlement::UiNativeTextAtlasCommitOutcome;
use crate::native::text_atlas::transaction::{
    UiNativeTextAtlasExternalOutcome, UiNativeTextAtlasPinRequest, UiNativeTextAtlasPinTransition,
};
use crate::native::text_atlas::{UiNativeTextAtlasDemand, UiNativeTextAtlasUpload};
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterDemandIdentity,
    UiGlyphRasterFractionalOrigin, UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterPalette,
    UiGlyphRasterSize, UiGlyphRasterSource, UiGlyphVariationCoordinates,
    UiQualifiedFontFaceIdentity, UiQualifiedTextLayoutIdentity, UiTextProfileGeneration,
};

fn key(glyph_id: u32, source: UiGlyphRasterSource) -> UiGlyphRasterKey {
    UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([1; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([2; 32], 0),
        glyph_id,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source,
        dpi_milli: 1_000,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap()
}

fn demand(key: UiGlyphRasterKey) -> UiNativeTextAtlasDemand {
    demand_with_extent(key, 2)
}

fn demand_with_extent(key: UiGlyphRasterKey, extent: u32) -> UiNativeTextAtlasDemand {
    let channels = match key.source() {
        UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap => 4,
        UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort => 1,
    };
    UiNativeTextAtlasDemand::from_native_geometry(
        UiGlyphRasterDemandIdentity::from_text_mechanics([7; 32]),
        key,
        extent,
        extent,
        u64::from(extent) * u64::from(extent) * channels,
    )
}

fn upload(key: UiGlyphRasterKey) -> UiNativeTextAtlasUpload {
    upload_with_extent(key, 2)
}

fn upload_with_extent(key: UiGlyphRasterKey, extent: u32) -> UiNativeTextAtlasUpload {
    let channels = match key.source() {
        UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap => 4,
        UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort => 1,
    };
    UiNativeTextAtlasUpload::from_text_mechanics(
        key,
        extent,
        extent,
        extent * channels,
        vec![0; (extent * extent * channels) as usize],
        [0; 32],
    )
}

#[test]
fn hit_miss_and_drop_are_atomic_and_alpha_color_never_alias() {
    let atlas = UiNativeTextAtlas::new();
    let alpha_key = key(1, UiGlyphRasterSource::AlphaOutline);
    let color_key = key(1, UiGlyphRasterSource::ColorOutline);
    let plan = atlas
        .plan_demands(&[demand(alpha_key)], &Default::default())
        .unwrap();
    assert_eq!(plan.miss_demands().len(), 1);
    let plan_snapshot = plan.snapshot();
    assert_eq!(plan_snapshot.demand_identity().digest(), [7; 32]);
    assert_eq!(plan_snapshot.predecessor_generation().get(), 1);
    assert_eq!(plan_snapshot.candidate_generation().get(), 2);
    assert_eq!(plan_snapshot.peak_entries(), 1);
    assert_eq!(plan_snapshot.peak_texel_bytes(), 1_048_576);
    assert_eq!(plan_snapshot.misses(), 1);
    assert_eq!(plan_snapshot.hits(), 0);
    assert_eq!(plan_snapshot.evictions(), 0);
    assert_eq!(plan_snapshot.staged_bytes(), 4);
    assert_eq!(plan_snapshot.physical_staged_bytes(), 512);
    assert_eq!(plan.predecessor_generation().get(), 1);
    assert_eq!(plan.staged_bytes(), 4);
    assert_eq!(plan.physical_staged_bytes(), 512);
    assert!(atlas.snapshot().reservation_active);
    assert_eq!(atlas.census().plans, 1);
    assert_eq!(atlas.census().reservations, 1);
    drop(plan);
    assert_eq!(atlas.snapshot().alpha_entries, 0);
    assert!(atlas.census().is_zero());
    let alpha_plan = atlas
        .plan_demands(&[demand(alpha_key)], &Default::default())
        .unwrap();
    assert!(matches!(
        atlas.settle(
            alpha_plan,
            &[upload(alpha_key)],
            UiNativeTextAtlasExternalOutcome::Submitted
        ),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let color_plan = atlas
        .plan_demands(&[demand(color_key)], &Default::default())
        .unwrap();
    assert!(matches!(
        atlas.settle(
            color_plan,
            &[upload(color_key)],
            UiNativeTextAtlasExternalOutcome::Submitted
        ),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let snapshot = atlas.snapshot();
    assert_eq!(snapshot.alpha_entries, 1);
    assert_eq!(snapshot.color_entries, 1);
    let hit = atlas
        .plan_demands(&[demand(alpha_key)], &Default::default())
        .unwrap();
    assert!(hit.miss_demands().is_empty());
    assert_eq!(hit.hit_keys(), &[alpha_key]);
    drop(hit);
}

#[test]
pub(super) fn pinned_entry_survives_page_saturation_and_unpinned_entry_is_evicted() {
    let atlas = UiNativeTextAtlas::new();
    let pinned_key = key(0, UiGlyphRasterSource::AlphaOutline);
    let first = atlas
        .plan_demands(&[demand_with_extent(pinned_key, 512)], &Default::default())
        .unwrap();
    assert!(matches!(
        atlas.settle(
            first,
            &[upload_with_extent(pinned_key, 512)],
            UiNativeTextAtlasExternalOutcome::Submitted
        ),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([9; 32]);
    let pin = UiNativeTextAtlasPinTransition::from_text_mechanics(
        [UiNativeTextAtlasPinRequest::from_text_mechanics(
            layout, pinned_key,
        )],
        [],
    );
    let pin_plan = atlas.plan_demands(&[], &pin).unwrap();
    assert!(matches!(
        atlas.settle(pin_plan, &[], UiNativeTextAtlasExternalOutcome::Submitted),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    for glyph in 1..=64 {
        let glyph_key = key(glyph, UiGlyphRasterSource::AlphaOutline);
        let plan = atlas
            .plan_demands(&[demand_with_extent(glyph_key, 512)], &Default::default())
            .unwrap();
        let _ = atlas.settle(
            plan,
            &[upload_with_extent(glyph_key, 512)],
            UiNativeTextAtlasExternalOutcome::Submitted,
        );
    }
    let replacement_key = key(100, UiGlyphRasterSource::AlphaOutline);
    let replacement = atlas
        .plan_demands(
            &[demand_with_extent(replacement_key, 512)],
            &Default::default(),
        )
        .unwrap();
    assert!(replacement
        .evicted_keys()
        .iter()
        .any(|evicted| *evicted != pinned_key));
    assert!(!replacement.evicted_keys().contains(&pinned_key));
}

#[test]
pub(super) fn settlement_rejects_duplicate_uploads_and_foreign_plans_without_mutation() {
    let first_key = key(201, UiGlyphRasterSource::AlphaOutline);
    let second_key = key(202, UiGlyphRasterSource::AlphaOutline);
    let first_atlas = UiNativeTextAtlas::new();
    let second_atlas = UiNativeTextAtlas::new();
    let plan = first_atlas
        .plan_demands(
            &[demand(first_key), demand(second_key)],
            &Default::default(),
        )
        .unwrap();
    assert_eq!(
        first_atlas.settle(
            plan,
            &[upload(first_key), upload(first_key)],
            UiNativeTextAtlasExternalOutcome::Submitted,
        ),
        UiNativeTextAtlasCommitOutcome::Denied(super::UiNativeTextAtlasDenial::RasterBatchMismatch,)
    );
    assert!(first_atlas.census().is_zero());

    let foreign_plan = first_atlas
        .plan_demands(&[demand(first_key)], &Default::default())
        .unwrap();
    assert_eq!(
        second_atlas.settle(
            foreign_plan,
            &[upload(first_key)],
            UiNativeTextAtlasExternalOutcome::Submitted,
        ),
        UiNativeTextAtlasCommitOutcome::Denied(super::UiNativeTextAtlasDenial::StalePlan)
    );
    assert!(!first_atlas.snapshot().reservation_active);
    assert!(second_atlas.census().is_zero());
}

#[test]
pub(super) fn releasing_one_of_two_layout_pins_keeps_the_shared_entry_protected() {
    let atlas = UiNativeTextAtlas::new();
    let pinned_key = key(301, UiGlyphRasterSource::AlphaOutline);
    let seed = atlas
        .plan_demands(&[demand_with_extent(pinned_key, 512)], &Default::default())
        .unwrap();
    assert!(matches!(
        atlas.settle(
            seed,
            &[upload_with_extent(pinned_key, 512)],
            UiNativeTextAtlasExternalOutcome::Submitted,
        ),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let first_layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([31; 32]);
    let second_layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([32; 32]);
    for layout in [first_layout, second_layout] {
        let transition = UiNativeTextAtlasPinTransition::from_text_mechanics(
            [UiNativeTextAtlasPinRequest::from_text_mechanics(
                layout, pinned_key,
            )],
            [],
        );
        let plan = atlas.plan_demands(&[], &transition).unwrap();
        assert!(matches!(
            atlas.settle(plan, &[], UiNativeTextAtlasExternalOutcome::Submitted),
            UiNativeTextAtlasCommitOutcome::Committed(_)
        ));
    }
    let pinned = |layout| {
        atlas.is_pinned(
            worth_ui_host_contract::UiGlyphRasterPinRequest::from_text_mechanics(
                layout, pinned_key,
            ),
        )
    };
    assert!(pinned(first_layout) && pinned(second_layout));
    assert_eq!(atlas.pin_count(), 2);
    let release = UiNativeTextAtlasPinTransition::from_text_mechanics(
        [],
        [UiNativeTextAtlasPinRequest::from_text_mechanics(
            first_layout,
            pinned_key,
        )],
    );
    let release_plan = atlas.plan_demands(&[], &release).unwrap();
    assert!(matches!(
        atlas.settle(
            release_plan,
            &[],
            UiNativeTextAtlasExternalOutcome::Submitted,
        ),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    assert!(!pinned(first_layout) && pinned(second_layout));
    assert_eq!(atlas.pin_count(), 1);
    for glyph in 400..=463 {
        let replacement_key = key(glyph, UiGlyphRasterSource::AlphaOutline);
        let plan = atlas
            .plan_demands(
                &[demand_with_extent(replacement_key, 512)],
                &Default::default(),
            )
            .unwrap();
        let _ = atlas.settle(
            plan,
            &[upload_with_extent(replacement_key, 512)],
            UiNativeTextAtlasExternalOutcome::Submitted,
        );
    }
    let final_plan = atlas
        .plan_demands(
            &[demand_with_extent(
                key(500, UiGlyphRasterSource::AlphaOutline),
                512,
            )],
            &Default::default(),
        )
        .unwrap();
    assert!(!final_plan.evicted_keys().contains(&pinned_key));
}

#[test]
pub(super) fn committed_hit_refreshes_recency_before_the_next_saturation_eviction() {
    let atlas = UiNativeTextAtlas::new();
    for glyph in 0..16 {
        let glyph_key = key(glyph, UiGlyphRasterSource::AlphaOutline);
        let plan = atlas
            .plan_demands(&[demand_with_extent(glyph_key, 512)], &Default::default())
            .unwrap();
        assert!(matches!(
            atlas.settle(
                plan,
                &[upload_with_extent(glyph_key, 512)],
                UiNativeTextAtlasExternalOutcome::Submitted,
            ),
            UiNativeTextAtlasCommitOutcome::Committed(_)
        ));
    }
    let recently_used = key(0, UiGlyphRasterSource::AlphaOutline);
    let hit = atlas
        .plan_demands(
            &[demand_with_extent(recently_used, 512)],
            &Default::default(),
        )
        .unwrap();
    assert!(hit.miss_demands().is_empty());
    assert!(matches!(
        atlas.settle(hit, &[], UiNativeTextAtlasExternalOutcome::Submitted),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let replacement = atlas
        .plan_demands(
            &[demand_with_extent(
                key(1_000, UiGlyphRasterSource::AlphaOutline),
                512,
            )],
            &Default::default(),
        )
        .unwrap();
    assert!(!replacement.evicted_keys().contains(&recently_used));
    assert!(replacement
        .evicted_keys()
        .contains(&key(1, UiGlyphRasterSource::AlphaOutline)));
}

#[test]
pub(super) fn released_pin_is_eligible_for_atomic_same_transaction_replacement() {
    let atlas = UiNativeTextAtlas::new();
    let old_key = key(1_001, UiGlyphRasterSource::AlphaOutline);
    let seed = atlas
        .plan_demands(&[demand_with_extent(old_key, 512)], &Default::default())
        .unwrap();
    assert!(matches!(
        atlas.settle(
            seed,
            &[upload_with_extent(old_key, 512)],
            UiNativeTextAtlasExternalOutcome::Submitted,
        ),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([77; 32]);
    let pin = UiNativeTextAtlasPinTransition::from_text_mechanics(
        [UiNativeTextAtlasPinRequest::from_text_mechanics(
            layout, old_key,
        )],
        [],
    );
    let pin_plan = atlas.plan_demands(&[], &pin).unwrap();
    assert!(matches!(
        atlas.settle(pin_plan, &[], UiNativeTextAtlasExternalOutcome::Submitted),
        UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    for glyph in 2_000..2_063 {
        let glyph_key = key(glyph, UiGlyphRasterSource::AlphaOutline);
        let plan = atlas
            .plan_demands(&[demand_with_extent(glyph_key, 512)], &Default::default())
            .unwrap();
        let _ = atlas.settle(
            plan,
            &[upload_with_extent(glyph_key, 512)],
            UiNativeTextAtlasExternalOutcome::Submitted,
        );
    }
    let replacement_key = key(3_000, UiGlyphRasterSource::AlphaOutline);
    let transition = UiNativeTextAtlasPinTransition::from_text_mechanics(
        [UiNativeTextAtlasPinRequest::from_text_mechanics(
            layout,
            replacement_key,
        )],
        [UiNativeTextAtlasPinRequest::from_text_mechanics(
            layout, old_key,
        )],
    );
    let replacement = atlas
        .plan_demands(&[demand_with_extent(replacement_key, 512)], &transition)
        .unwrap();
    assert!(replacement.evicted_keys().contains(&old_key));
}
