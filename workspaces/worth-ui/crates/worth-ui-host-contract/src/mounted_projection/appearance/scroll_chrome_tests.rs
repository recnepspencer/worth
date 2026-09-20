use super::*;

fn owner() -> crate::UiMountedInstanceIdentity {
    crate::UiMountedInstanceIdentity::mint_unbound().expect("an unbound mounted instance")
}

fn rect() -> crate::UiAppearanceAllocationBounds {
    crate::UiAppearanceAllocationBounds::new(100_000, 20_000, 6_000, 48_000)
        .expect("a nonempty chrome rectangle")
}

fn radii(rect: crate::UiAppearanceAllocationBounds) -> crate::UiAppearanceNormalizedLogicalRadii {
    crate::UiAppearanceNormalizedLogicalRadii::normalize(
        rect,
        [crate::UiAppearanceLogicalLength::new(3_000).expect("a declared radius"); 4],
    )
}

fn input(
    owner_instance: crate::UiMountedInstanceIdentity,
    semantic_surface: crate::UiSemanticSurfaceIdentity,
    part: UiMountedScrollChromePart,
) -> UiMountedScrollChromeCompletionInput {
    let rect = rect();
    UiMountedScrollChromeCompletionInput {
        identity: UiMountedScrollChromeIdentity::from_runtime_mounting(
            owner_instance,
            UiMountedScrollChromeAxis::Block,
            part,
        ),
        semantic_surface,
        rect,
        clip: crate::UiAppearanceClip::new(0, 0, 200_000, 200_000).expect("a region clip"),
        background: crate::UiMountedAppearanceColor::from_straight_srgba([90, 90, 90, 255]),
        radii: radii(rect),
        opacity: crate::UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
        attribution: UiMountedScrollChromeAppearanceAttribution::from_runtime_transport(
            semantic_surface,
            owner_instance,
            0x5041_494e_5445_4400,
            0x4841_5645_5245_4400,
        )
        .expect("nonzero chrome attribution digests"),
    }
}

/// The mechanic carries its own non-node attribution and paints nothing the
/// pointer is answered from.
#[test]
fn scroll_chrome_completion_uses_distinct_non_node_attribution() {
    let owner_instance = owner();
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("a surface");
    let mechanic = UiMountedScrollChromeMechanic::complete_from_runtime_mounting(input(
        owner_instance,
        surface,
        UiMountedScrollChromePart::Thumb,
    ))
    .expect("a well formed chrome mechanic completes");
    assert_eq!(mechanic.identity().owner_instance(), owner_instance);
    assert_eq!(mechanic.identity().part(), UiMountedScrollChromePart::Thumb);
    assert_eq!(mechanic.attribution().role_digest(), 0x5041_494e_5445_4400);
    assert!(!mechanic.participates_in_hit_testing());
}

/// A zero digest names no role and no resolved appearance, so it cannot be
/// transported as one.
#[test]
fn scroll_chrome_attribution_denies_zero_digests() {
    let owner_instance = owner();
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("a surface");
    assert!(
        UiMountedScrollChromeAppearanceAttribution::from_runtime_transport(
            surface,
            owner_instance,
            0,
            1
        )
        .is_none()
    );
    assert!(
        UiMountedScrollChromeAppearanceAttribution::from_runtime_transport(
            surface,
            owner_instance,
            1,
            0
        )
        .is_none()
    );
}

/// Attribution that was completed for another surface or another occurrence is
/// refused rather than silently re-pointed.
#[test]
fn scroll_chrome_completion_denies_foreign_attribution() {
    let owner_instance = owner();
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("a surface");
    let mut foreign_surface = input(owner_instance, surface, UiMountedScrollChromePart::Track);
    foreign_surface.semantic_surface =
        crate::UiSemanticSurfaceIdentity::mint_unbound().expect("another surface");
    assert_eq!(
        UiMountedScrollChromeMechanic::complete_from_runtime_mounting(foreign_surface),
        Err(UiMountedScrollChromeCompletionDenial::AttributionSurfaceMismatch)
    );

    let mut foreign_owner = input(owner_instance, surface, UiMountedScrollChromePart::Track);
    foreign_owner.identity = UiMountedScrollChromeIdentity::from_runtime_mounting(
        owner(),
        UiMountedScrollChromeAxis::Block,
        UiMountedScrollChromePart::Track,
    );
    assert_eq!(
        UiMountedScrollChromeMechanic::complete_from_runtime_mounting(foreign_owner),
        Err(UiMountedScrollChromeCompletionDenial::AttributionOwnerMismatch)
    );
}

/// Radii normalized against a different rectangle would round a bar that is no
/// longer that size.
#[test]
fn scroll_chrome_completion_denies_radii_from_another_rectangle() {
    let owner_instance = owner();
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("a surface");
    let mut input = input(owner_instance, surface, UiMountedScrollChromePart::Thumb);
    input.radii = radii(
        crate::UiAppearanceAllocationBounds::new(0, 0, 12_000, 12_000).expect("another rectangle"),
    );
    assert_eq!(
        UiMountedScrollChromeMechanic::complete_from_runtime_mounting(input),
        Err(UiMountedScrollChromeCompletionDenial::RadiiRectangleMismatch)
    );
}

/// A viewport too small for its own gutter shows no chrome. That absence is a
/// denial here rather than a rectangle painted at zero coverage.
#[test]
fn scroll_chrome_completion_denies_a_clip_that_excludes_the_rectangle() {
    let owner_instance = owner();
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("a surface");
    let mut input = input(owner_instance, surface, UiMountedScrollChromePart::Track);
    input.clip = crate::UiAppearanceClip::new(0, 0, 50_000, 50_000).expect("a narrow clip");
    assert_eq!(
        UiMountedScrollChromeMechanic::complete_from_runtime_mounting(input),
        Err(UiMountedScrollChromeCompletionDenial::ClipExcludesRectangle)
    );
}

/// The paint digest moves with every field a host paints from and stays put
/// when only transport lineage changes.
#[test]
fn scroll_chrome_paint_digest_tracks_painted_fields_only() {
    let owner_instance = owner();
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("a surface");
    let resting = UiMountedScrollChromeMechanic::complete_from_runtime_mounting(input(
        owner_instance,
        surface,
        UiMountedScrollChromePart::Thumb,
    ))
    .expect("the resting thumb completes");

    let mut relineaged = input(owner_instance, surface, UiMountedScrollChromePart::Thumb);
    relineaged.attribution = UiMountedScrollChromeAppearanceAttribution::from_runtime_transport(
        surface,
        owner_instance,
        0x5041_494e_5445_4400,
        0x4845_4c44_0000_0001,
    )
    .expect("nonzero digests");
    let relineaged = UiMountedScrollChromeMechanic::complete_from_runtime_mounting(relineaged)
        .expect("a relineaged thumb completes");
    assert_eq!(resting.paint_digest(), relineaged.paint_digest());

    let mut hovered = input(owner_instance, surface, UiMountedScrollChromePart::Thumb);
    hovered.background = crate::UiMountedAppearanceColor::from_straight_srgba([140, 140, 140, 255]);
    let hovered = UiMountedScrollChromeMechanic::complete_from_runtime_mounting(hovered)
        .expect("a hovered thumb completes");
    assert_ne!(resting.paint_digest(), hovered.paint_digest());

    let mut moved = input(owner_instance, surface, UiMountedScrollChromePart::Thumb);
    moved.rect = crate::UiAppearanceAllocationBounds::new(100_000, 60_000, 6_000, 48_000)
        .expect("the same thumb further down");
    moved.radii = radii(moved.rect);
    let moved = UiMountedScrollChromeMechanic::complete_from_runtime_mounting(moved)
        .expect("a moved thumb completes");
    assert_ne!(resting.paint_digest(), moved.paint_digest());
}

/// The track is painted under the thumb, and the two parts are two identities
/// on the same occurrence.
#[test]
fn scroll_chrome_parts_order_the_track_under_the_thumb() {
    assert_eq!(
        UiMountedScrollChromePart::PAINT_ORDER,
        [
            UiMountedScrollChromePart::Track,
            UiMountedScrollChromePart::Thumb
        ]
    );
    assert!(
        UiMountedScrollChromePart::Track.paint_ordinal()
            < UiMountedScrollChromePart::Thumb.paint_ordinal()
    );
    let owner_instance = owner();
    let track = UiMountedScrollChromeIdentity::from_runtime_mounting(
        owner_instance,
        UiMountedScrollChromeAxis::Block,
        UiMountedScrollChromePart::Track,
    );
    let thumb = UiMountedScrollChromeIdentity::from_runtime_mounting(
        owner_instance,
        UiMountedScrollChromeAxis::Block,
        UiMountedScrollChromePart::Thumb,
    );
    let inline_track = UiMountedScrollChromeIdentity::from_runtime_mounting(
        owner_instance,
        UiMountedScrollChromeAxis::Inline,
        UiMountedScrollChromePart::Track,
    );
    assert_ne!(track, thumb);
    assert_ne!(track, inline_track);
}
