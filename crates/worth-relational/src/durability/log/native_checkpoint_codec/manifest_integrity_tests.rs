use super::*;

#[test]
fn sparse_page_membership_accepts_late_holes_and_refuses_duplicate_or_forged_ids() {
    use crate::durability::data::RelationalCheckpointPage as Page;
    use crate::durability::log::native_checkpoint_codec::member_frame::locator;
    let odd = Page::from_ids([1, 3], locator(b"first-actual-page")).unwrap();
    let even = Page::from_ids([2], locator(b"late-publication-page")).unwrap();
    let pages = [odd, even];
    validate_pages(&pages).unwrap();
    assert_eq!(Page::find(&pages, 2), Some(&even));
    assert_eq!(Page::find(&pages, 3), Some(&odd));
    assert!(Page::find(&pages, 4).is_none());
    let duplicate = Page::from_ids([3], locator(b"competing-page")).unwrap();
    assert!(validate_pages(&[odd, even, duplicate]).is_err());
    let mut forged = odd;
    forged.membership[0] |= 1 << 4;
    assert!(
        validate_pages(&[forged]).is_err(),
        "the declared endpoints must cover the exact bits"
    );
    let mut overflow = Page::from_ids([u64::MAX - 2], locator(b"last-range")).unwrap();
    overflow.membership[3] |= 1 << 63;
    assert!(
        validate_pages(&[overflow]).is_err(),
        "a hostile high bit must refuse without arithmetic overflow"
    );
}
