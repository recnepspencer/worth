//! Pure Home2 framing, not selected membership or Native/World authority.
use super::*;

fn literal() -> Vec<u8> {
    vec![
        87, 65, 80, 72, 67, 75, 80, 49, 2, 0, 0, 0, 2, 0, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 4, 0, 1, 0,
        0, 0, 0, 0, 0, 0, 0xaa, 0xbb, 0xcc,
    ]
}
#[test]
fn literal_two_owner_directory_preserves_both_opaque_sections() {
    let bytes = literal();
    let sections = decode(&bytes).unwrap();
    assert_eq!(sections.relational, &[0xaau8, 0xbb]);
    assert_eq!(sections.world, &[0xccu8]);
    let mut written = Vec::new();
    emit(&[0xaa, 0xbb], &[0xcc], &mut written);
    assert_eq!(written, bytes);
    assert_eq!(HEADER_BYTES, 34);
}
#[test]
fn old_missing_empty_duplicate_and_unrecognized_owners_refuse() {
    let mut old = literal();
    old[8] = 1;
    assert!(matches!(
        decode(&old),
        Err(HomeManifestDenial::UnsupportedVersion(_))
    ));
    let mut missing = literal();
    missing[12] = 1;
    assert!(matches!(
        decode(&missing),
        Err(HomeManifestDenial::Malformed)
    ));
    let mut empty = literal();
    empty[26] = 0;
    empty.pop();
    assert!(matches!(decode(&empty), Err(HomeManifestDenial::Malformed)));
    let mut duplicate = literal();
    duplicate[24] = 1;
    assert!(matches!(
        decode(&duplicate),
        Err(HomeManifestDenial::Malformed)
    ));
    let mut unsupported = literal();
    unsupported[24] = 2;
    assert!(matches!(
        decode(&unsupported),
        Err(HomeManifestDenial::UnsupportedOwner(
            DurableOwnerFamily::Signal
        ))
    ));
    unsupported[24..26].copy_from_slice(&u16::MAX.to_le_bytes());
    assert!(matches!(
        decode(&unsupported),
        Err(HomeManifestDenial::UnknownOwner { tag: u16::MAX })
    ));
}
#[test]
fn lengths_are_checked_against_the_actual_remaining_payload() {
    let mut huge = literal();
    huge[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(
        decode(&huge),
        Err(HomeManifestDenial::Malformed | HomeManifestDenial::SizeLimit)
    ));
    let mut extra = literal();
    extra.push(0);
    assert!(matches!(decode(&extra), Err(HomeManifestDenial::Malformed)));
    let mut truncated = literal();
    truncated.pop();
    assert!(matches!(
        decode(&truncated),
        Err(HomeManifestDenial::Malformed)
    ));
}
