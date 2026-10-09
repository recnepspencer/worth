//! Exact tier authorization checks consume witnesses without materialization.

use super::*;

fn witness(start: u64, end: u64, identity: u8, payload: u8) -> TierEpochWalFrameWitnessV1 {
    TierEpochWalFrameWitnessV1::new(start, end, [identity; 32], [payload; 32]).unwrap()
}

fn witnesses(
    frames: &[TierEpochWalFrameWitnessV1],
) -> impl Iterator<Item = Option<TierEpochWalFrameWitnessV1>> + Clone + '_ {
    frames.iter().copied().map(Some)
}

#[test]
fn selected_pair_requires_exact_interval_and_payload_not_an_overlapping_member() {
    let expected = witness(10, 20, 1, 2);
    assert_eq!(
        exact_selected_frame(witnesses(&[expected]), 10, 20, [2; 32]),
        Some(expected)
    );
    for frames in [
        &[witness(10, 19, 1, 2)][..],
        &[witness(11, 20, 1, 2)][..],
        &[witness(10, 20, 1, 3)][..],
        &[expected, expected][..],
    ] {
        assert!(exact_selected_frame(witnesses(frames), 10, 20, [2; 32]).is_none());
    }
}

#[test]
fn checkpoint_witness_denies_torn_or_conflicting_retained_c9_member() {
    let expected = witness(100, 110, 4, 5);
    assert!(retained_witness_agrees(witnesses(&[]), expected));
    assert!(retained_witness_agrees(witnesses(&[expected]), expected));
    for frames in [
        &[witness(100, 109, 4, 5)][..],
        &[witness(100, 110, 9, 5)][..],
        &[witness(100, 110, 4, 9)][..],
        &[expected, expected][..],
    ] {
        assert!(!retained_witness_agrees(witnesses(frames), expected));
    }
}

#[test]
fn malformed_witness_is_denied_even_outside_the_requested_interval() {
    let expected = witness(10, 20, 1, 2);
    let frames = [Some(witness(100, 110, 4, 5)), None, Some(expected)];
    assert!(exact_selected_frame(frames.into_iter(), 10, 20, [2; 32]).is_none());
    assert!(!retained_witness_agrees(frames.into_iter(), expected));
}

#[test]
fn folded_checkpoint_denies_conflicting_selected_tail_activation() {
    let intent = TierEpochActivationV1::intent(
        [1; 16], [2; 16], 3, [4; 32], [5; 32], 7, 4, [6; 32], 4096, 8,
    )
    .unwrap();
    let intent_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
    let completed_digest: [u8; 32] = Sha256::digest(intent.completed().encode()).into();
    let first = TierEpochWalFrameWitnessV1::new(10, 20, [1; 32], intent_digest).unwrap();
    let second = TierEpochWalFrameWitnessV1::new(30, 40, [2; 32], completed_digest).unwrap();
    let matches =
        |frames: &[TierEpochWalFrameWitnessV1], observed_intent, intent_range, completed_range| {
            selected_pair_matches_certificate(
                witnesses(frames),
                intent,
                first,
                second,
                observed_intent,
                intent_range,
                completed_range,
            )
        };
    assert!(matches(&[first, second], intent, (10, 20), Some((30, 40))));
    assert!(!matches(&[first, second], intent, (10, 20), None));
    assert!(!matches(&[first, second], intent, (10, 20), Some((41, 50))));
    assert!(!matches(&[first, second], intent, (11, 20), Some((30, 40))));
    assert!(!matches(
        &[first, second, second],
        intent,
        (10, 20),
        Some((30, 40))
    ));
    assert!(!matches(
        &[first, witness(30, 40, 9, 9)],
        intent,
        (10, 20),
        Some((30, 40)),
    ));
    assert!(!matches(
        &[first, second],
        intent.completed(),
        (10, 20),
        Some((30, 40)),
    ));
}
