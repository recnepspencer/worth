//! Pure three-owner framing, not selected membership or owner authority.
use super::*;

fn literal() -> Vec<u8> {
    vec![
        87, 65, 80, 72, 67, 75, 80, 49, 3, 0, 0, 0, 3, 0, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 4, 0, 1, 0,
        0, 0, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0xaa, 0xbb, 0xcc, 0xdd,
    ]
}
#[test]
fn literal_three_owner_directory_preserves_all_opaque_sections() {
    let bytes = literal();
    let sections = decode(&bytes).unwrap();
    assert_eq!(sections.relational, &[0xaau8, 0xbb]);
    assert_eq!(sections.world, &[0xccu8]);
    assert_eq!(sections.signal, &[0xddu8]);
    let mut written = Vec::new();
    emit(&[0xaa, 0xbb], &[0xcc], &[0xdd], &mut written);
    assert_eq!(written, bytes);
    assert_eq!(HEADER_BYTES, 44);
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
    unsupported[24] = 3;
    assert!(matches!(
        decode(&unsupported),
        Err(HomeManifestDenial::UnsupportedOwner(
            DurableOwnerFamily::Bridge
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
