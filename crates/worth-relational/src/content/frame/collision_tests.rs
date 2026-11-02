use super::*;

#[test]
fn equal_claimed_identity_with_unequal_actual_bytes_is_an_internal_conflict() {
    // Corrupt a sealed admission result's body while preserving its claimed id.
    // The production caller charges each chunk before this independent exact check.
    let make = |byte| {
        RelationalContentFrame(Arc::new(AdmittedFrame {
            record: RelationalContentRecord {
                identity: ContentRefId([9; 32]),
                backing: Arc::new(FrameBacking {
                    bytes: vec![byte],
                    memory: None,
                }),
            },
            format: ContentFormat::new("sample-grid", 1).unwrap(),
            payload_offset: 0,
            _metadata: None,
        }))
    };
    let left = make(1);
    let right = make(2);
    assert_eq!(left.identity(), right.identity());
    assert_eq!(
        certify_equal_chunk(left.bytes(), right.bytes()),
        Err(Denial::ContentIdentityConflict)
    );
    assert_eq!(certify_equal_chunk(left.bytes(), left.bytes()), Ok(()));
}
