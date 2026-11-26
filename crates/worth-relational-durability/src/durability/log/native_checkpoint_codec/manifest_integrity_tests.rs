use super::*;

#[test]
fn sparse_page_membership_accepts_late_holes_and_refuses_duplicate_or_forged_ids() {
    use crate::durability::data::RelationalCheckpointPage as Page;
    use worth_relational_storage::internal_port::storage::data::member_frame::locator;
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

#[test]
fn record_identity_validation_refuses_unsorted_orphan_overflow_and_mismatched_roots() {
    use worth_relational_contracts::internal_port::identity::data::primitives::PartitionId;
    use worth_relational_storage::internal_port::storage::data::member_frame::locator;
    use worth_relational_storage::internal_port::storage::data::record_identity_pages::{
        PagedRecordIdentity as Paged, PagedRecordIdentityRoots as Roots,
    };
    use worth_relational_storage::internal_port::storage::residency::chunked::paged_directory::PagedDirectoryRoot as Root;

    validate_record_identity(&Paged::default()).unwrap();
    let mut pages = [locator(b"id-page-a"), locator(b"id-page-b")];
    pages.sort_unstable();
    let [low, high] = pages;
    let r = |root_page, entry_count| Root {
        root_page,
        entry_count,
    };
    let valid = Paged {
        roots: Roots {
            generation_high_water: Some(r(low, 2)),
            reusable_slots: None,
            append_frontiers: Some(r(high, 1)),
            pending_reservations: None,
            partition_high_water: Some(PartitionId(0)),
        },
        pages: vec![low, high],
    };
    validate_record_identity(&valid).unwrap();
    let bad = |mutate: &dyn Fn(&mut Paged)| {
        let mut p = valid.clone();
        mutate(&mut p);
        assert!(validate_record_identity(&p).is_err());
    };
    bad(&|p| p.pages = vec![high, low]);
    bad(&|p| p.pages = vec![low, low]);
    bad(&|p| p.roots.generation_high_water = Some(r(low, 0)));
    bad(&|p| p.roots.reusable_slots = Some(r(locator(b"orphan"), 1)));
    bad(&|p| p.roots.generation_high_water = Some(r(low, u64::MAX)));
    bad(&|p| p.roots.partition_high_water = None);
    assert!(validate_record_identity(&Paged {
        roots: Roots::default(),
        pages: vec![low]
    })
    .is_err());
    let mut stray = Paged::default();
    stray.roots.partition_high_water = Some(PartitionId(0));
    assert!(validate_record_identity(&stray).is_err());
}
