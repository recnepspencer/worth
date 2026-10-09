use super::*;

#[test]
fn borrowed_head_view_validates_late_branch_reference_before_exposing_items() {
    let left = ReleaseCustodyHeadBlockV1::leaf(9, 8, 1, vec![entry(1, 1)], format()).unwrap();
    let right = ReleaseCustodyHeadBlockV1::leaf(9, 8, 2, vec![entry(2, 1)], format()).unwrap();
    let branch = ReleaseCustodyHeadBlockV1::branch(
        9,
        8,
        3,
        1,
        vec![left.reference(format()), right.reference(format())],
        format(),
    )
    .unwrap();
    let original = branch.reference(format());
    let encoded = branch.encode(format());
    let (view, _) = ReleaseCustodyHeadBlockViewV1::decode(&encoded, original, 9).unwrap();
    assert_eq!(
        view.children().unwrap().collect::<Vec<_>>().as_slice(),
        branch.children().unwrap()
    );

    let mut payload = encoded[48..].to_vec();
    payload[40 + ReleaseCustodyHeadBlockReferenceV1::ENCODED_BYTES + 18] = 1;
    let malformed = encode_durable_frame_schema(
        DurableFrameKind::ReleaseCustodyHeadBlock,
        format(),
        3,
        &payload,
        2,
    );
    let reference = reference_with_frame_sha(original, &malformed);
    assert!(matches!(
        ReleaseCustodyHeadBlockViewV1::decode(&malformed, reference, 9),
        Err(ReleaseCustodyHeadDenial::Malformed)
    ));
    assert_eq!(
        ReleaseCustodyHeadBlockV1::decode(&malformed, reference, 9),
        Err(ReleaseCustodyHeadDenial::Malformed)
    );
}
