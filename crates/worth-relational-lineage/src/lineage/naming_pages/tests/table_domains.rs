//! Naming pages isolate table domains: `seal_pages` never puts rows of two
//! domains on one physical page, makes no reads across domains, and
//! `validate_pages` refuses multi-domain or unknown-tag descriptors
//! read-free. Every domain is one contiguous tag interval, so equal first
//! and last domains prove a descriptor holds no other domain's rows.

use std::sync::Arc;

use worth_relational_contracts::internal_port::history::data::coordinates::BranchId;

use super::super::builder::seal_pages;
use super::super::store::ColdNamingStore;
use super::super::tables::{
    table_domain, GROUP_LIVE, GROUP_MEMBERS, INDEX_NAME, LINEAGE_HOLDER, MAX_INLINE_VALUE_BYTES,
    VALUE_CHUNK,
};
use super::super::{validate_pages, BranchNamingRoot, CheckpointNamingState};
use super::{CollectedMembers, MemSource};

#[test]
fn naming_pages_isolate_domains_without_cross_domain_reads_and_refuse_mixed_descriptors() {
    let k_naming = vec![LINEAGE_HOLDER, 1, 2, 3];
    let k_index = vec![INDEX_NAME, 1, 2, 3];
    let k_group = vec![GROUP_LIVE, 1, 2, 3];

    let mut sink1 = CollectedMembers::default();
    let big = vec![0x77u8; MAX_INLINE_VALUE_BYTES + 128];
    let rows1 = vec![(k_naming, Some(vec![1; 8])), (k_group, Some(big))];
    let pages1 = seal_pages(None, rows1, &mut sink1).unwrap();
    assert_eq!(pages1.len(), 3);
    assert!(pages1
        .iter()
        .all(|p| table_domain(p.first[0]) == table_domain(p.last[0])));
    assert_eq!(table_domain(pages1[0].first[0]), Some(1));
    assert_eq!(table_domain(pages1[1].first[0]), Some(3));
    assert_eq!(pages1[2].first[0], VALUE_CHUNK);
    assert!(validate_pages(&pages1).is_ok());

    let source = MemSource::from_sink(sink1);
    let root = BranchNamingRoot {
        branch: BranchId("main".into()),
        naming: None,
        index: None,
        groups: None,
        pages: pages1.clone(),
    };
    let state = CheckpointNamingState {
        branches: vec![root],
        group_log: Vec::new(),
        group_log_through: None,
    };
    let store = ColdNamingStore::open(Arc::new(state), source.clone());
    let mut sink2 = CollectedMembers::default();
    let rows2 = vec![(k_index.clone(), Some(vec![2; 8]))];
    let pages2 = seal_pages(Some((&store, &pages1)), rows2, &mut sink2).unwrap();
    assert_eq!(source.read_count(), 0);
    assert_eq!(pages2.len(), 4);
    assert_eq!(pages2[0], pages1[0]);
    assert_eq!(table_domain(pages2[1].first[0]), Some(2));
    assert_eq!(pages2[2], pages1[1]);
    assert_eq!(pages2[3], pages1[2]);
    assert!(validate_pages(&pages2).is_ok());

    // Synthetic mixed-domain descriptor refuses read-free.
    let mut mixed = pages1[0].clone();
    mixed.last = k_index;
    mixed.entries = 2;
    assert!(validate_pages(&[mixed]).is_err());

    // Unknown table tag descriptor refuses read-free.
    let mut unknown = pages1[0].clone();
    unknown.first = vec![u8::MAX, 1];
    unknown.last = vec![u8::MAX, 1];
    unknown.entries = 1;
    assert!(validate_pages(&[unknown]).is_err());
}

/// Every domain is one contiguous tag interval, so a descriptor whose first
/// and last keys share a domain can hold no row of another domain: group
/// member rows (tag 0) never share a page, or a descriptor, with group
/// headers (tag 11) across the naming and index tables between them.
#[test]
fn group_member_rows_keep_their_own_contiguous_domain() {
    let members = vec![GROUP_MEMBERS, 1, 2, 3];
    let headers = vec![GROUP_LIVE, 1, 2, 3];
    let mut sink = CollectedMembers::default();
    let rows = vec![(members, Some(vec![1; 8])), (headers, Some(vec![2; 8]))];
    let pages = seal_pages(None, rows, &mut sink).unwrap();
    assert_eq!(
        pages.len(),
        2,
        "member rows and headers seal on separate pages"
    );
    assert_eq!(
        (pages[0].first[0], pages[0].last[0]),
        (GROUP_MEMBERS, GROUP_MEMBERS)
    );
    assert_eq!(
        (pages[1].first[0], pages[1].last[0]),
        (GROUP_LIVE, GROUP_LIVE)
    );
    let mut spanning = pages[0].clone();
    spanning.last = pages[1].last.clone();
    spanning.entries = 2;
    assert!(
        validate_pages(&[spanning]).is_err(),
        "a descriptor spanning the naming and index tables refuses read-free"
    );
    for domain in (0..=u8::MAX).filter_map(table_domain) {
        let tags: Vec<u8> = (0..=u8::MAX)
            .filter(|tag| table_domain(*tag) == Some(domain))
            .collect();
        let span = usize::from(tags[tags.len() - 1] - tags[0]) + 1;
        assert_eq!(
            span,
            tags.len(),
            "domain {domain} is not one contiguous tag interval"
        );
    }
}
