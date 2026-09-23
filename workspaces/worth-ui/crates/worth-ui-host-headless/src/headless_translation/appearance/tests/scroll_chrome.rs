//! Chrome reaches the headless evidence surface and can be read back as pixels.

use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceClip, UiAppearanceLogicalLength,
    UiAppearanceNormalizedLogicalRadii, UiMountedAppearanceColor, UiMountedAppearanceFrame,
    UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange, UiMountedAppearanceWork,
    UiMountedAppearanceWorkPosture, UiMountedInstanceIdentity, UiMountedOverlayOrderMechanic,
    UiMountedPresentationOpacity, UiMountedScrollChromeAppearanceAttribution,
    UiMountedScrollChromeAxis, UiMountedScrollChromeCompletionInput, UiMountedScrollChromeIdentity,
    UiMountedScrollChromeMechanic, UiMountedScrollChromePart, UiSemanticSurfaceIdentity,
};

const TRACK_RGBA: [u8; 4] = [40, 40, 44, 255];
const THUMB_RGBA: [u8; 4] = [160, 160, 168, 255];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

struct ChromeFixture {
    surface: UiSemanticSurfaceIdentity,
    owner: UiMountedInstanceIdentity,
}

fn chrome(
    fixture: &ChromeFixture,
    part: UiMountedScrollChromePart,
    rect: [i32; 4],
    radius: i32,
    rgba: [u8; 4],
) -> UiMountedAppearanceMechanic {
    let rect = UiAppearanceAllocationBounds::new(rect[0], rect[1], rect[2] as u32, rect[3] as u32)
        .unwrap();
    UiMountedAppearanceMechanic::ScrollChrome(
        UiMountedScrollChromeMechanic::complete_from_runtime_mounting(
            UiMountedScrollChromeCompletionInput {
                identity: UiMountedScrollChromeIdentity::from_runtime_mounting(
                    fixture.owner,
                    UiMountedScrollChromeAxis::Block,
                    part,
                ),
                semantic_surface: fixture.surface,
                rect,
                clip: UiAppearanceClip::new(0, 0, 768_000, 269_000).unwrap(),
                background: UiMountedAppearanceColor::from_straight_srgba(rgba),
                radii: UiAppearanceNormalizedLogicalRadii::normalize(
                    rect,
                    [UiAppearanceLogicalLength::new(radius).unwrap(); 4],
                ),
                opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
                attribution: UiMountedScrollChromeAppearanceAttribution::from_runtime_transport(
                    fixture.surface,
                    fixture.owner,
                    0x5343_524f_4c4c_0001,
                    0x5041_5254_0000_0001 + u64::from(part.paint_ordinal()),
                )
                .unwrap(),
            },
        )
        .unwrap(),
    )
}

/// The gutter of a RecentActivity-shaped region: a 12 point gutter at the
/// inline end of a 768 x 269 point viewport, carrying a 6 point thumb centred
/// in it.
fn frame(fixture: &ChromeFixture) -> UiMountedAppearanceFrame {
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    UiMountedAppearanceFrame::from_runtime_mounting(
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        fixture.surface,
        [
            chrome(
                fixture,
                UiMountedScrollChromePart::Track,
                [756_000, 0, 12_000, 257_000],
                0,
                TRACK_RGBA,
            ),
            chrome(
                fixture,
                UiMountedScrollChromePart::Thumb,
                [759_000, 0, 6_000, 102_000],
                3_000,
                THUMB_RGBA,
            ),
        ],
        UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
            fixture.surface,
            presentation,
            0,
            0,
            [],
        )
        .unwrap(),
    )
    .unwrap()
}

fn transcript(
    fixture: &ChromeFixture,
) -> crate::headless_transcript::appearance::UiHeadlessAppearanceWorkTranscript {
    let successor = frame(fixture);
    let inserts = successor
        .mechanics()
        .iter()
        .cloned()
        .map(UiMountedAppearanceMechanicChange::Insert)
        .collect::<Vec<_>>();
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Initial,
        None,
        None,
        successor,
        inserts,
        [],
        true,
    )
    .unwrap();
    super::super::super::translate_appearance_fragment_work(&work).unwrap()
}

fn fixture() -> ChromeFixture {
    ChromeFixture {
        surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        owner: UiMountedInstanceIdentity::mint_unbound().unwrap(),
    }
}

/// The two parts survive translation as chrome, in paint order, and are not
/// mistaken for authored surfaces.
#[test]
fn chrome_translates_as_two_parts_of_one_occurrence() {
    let fixture = fixture();
    let transcript = transcript(&fixture);
    let mechanics = transcript.successor().mechanics();
    assert_eq!(mechanics.len(), 2);
    let parts = mechanics
        .iter()
        .map(|mechanic| match mechanic {
            crate::headless_transcript::appearance::UiHeadlessAppearanceMechanic::ScrollChrome(
                chrome,
            ) => chrome.identity().part(),
            other => panic!("chrome translated as {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        parts,
        [
            UiMountedScrollChromePart::Track,
            UiMountedScrollChromePart::Thumb
        ]
    );
}

/// The evidence surface reports the thumb where the thumb is, the track where
/// only the track is, and nothing outside the gutter.
#[test]
fn chrome_pixels_read_back_thumb_over_track_and_nothing_outside_the_gutter() {
    let fixture = fixture();
    let transcript = transcript(&fixture);
    let successor = transcript.successor();
    let sample = |x: i64, y: i64| {
        successor
            .reference_scroll_chrome_at(fixture.owner, x, y)
            .straight_srgba()
    };
    // Inside the thumb.
    assert_eq!(sample(762_000, 50_000), THUMB_RGBA);
    // In the gutter beside the thumb: the track alone.
    assert_eq!(sample(757_000, 50_000), TRACK_RGBA);
    // Below the thumb, still on the track.
    assert_eq!(sample(762_000, 200_000), TRACK_RGBA);
    // Left of the gutter entirely: scrolled content, no chrome.
    assert_eq!(sample(400_000, 50_000), TRANSPARENT);
    // Past the end of the track, inside the clip: nothing.
    assert_eq!(sample(762_000, 260_000), TRANSPARENT);
}

/// A point in a cut corner of the rounded thumb reports the track beneath it,
/// which is how the rounding is proved rather than asserted.
#[test]
fn chrome_thumb_corner_radius_shows_the_track_beneath_it() {
    let fixture = fixture();
    let transcript = transcript(&fixture);
    assert_eq!(
        transcript
            .successor()
            .reference_scroll_chrome_at(fixture.owner, 759_100, 100)
            .straight_srgba(),
        TRACK_RGBA,
        "the thumb inline-start corner is cut away"
    );
}

/// Chrome that names another occurrence is not this occurrence's chrome.
#[test]
fn chrome_evidence_is_keyed_by_the_occurrence_that_owns_the_gutter() {
    let fixture = fixture();
    let transcript = transcript(&fixture);
    let stranger = UiMountedInstanceIdentity::mint_unbound().unwrap();
    assert_eq!(
        transcript
            .successor()
            .reference_scroll_chrome_at(stranger, 762_000, 50_000)
            .straight_srgba(),
        TRANSPARENT
    );
}
