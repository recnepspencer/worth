//! The whole-result footprint keeps every distinct retained fact through
//! deduplication: one member's group membership never stands in for
//! another's.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::persistent_group::{
    PersistentGroupAnswer, PersistentGroupRead, PersistentGroupSubject,
};
use crate::domain_computation::primary_graph::application_query::observed_source::member_groups::WorthQueryObservedMemberGroups;

fn member(byte: u8, group: u8) -> Fact {
    WorthQueryObservedMemberGroups {
        member: [byte; 32],
        kind: "Member".into(),
        groups: vec![("Loop".into(), [group; 32])],
    }
    .fact()
}

fn group_read(byte: u8) -> Fact {
    Fact::PersistentGroup {
        read: PersistentGroupRead {
            group_kind: "Loop".into(),
            member_kind: "Member".into(),
            subject: PersistentGroupSubject::Member([byte; 32]),
        },
        answer: PersistentGroupAnswer::Member(None),
    }
}

#[test]
fn every_named_row_members_group_membership_survives_deduplication() {
    let mut facts = vec![member(2, 9), member(1, 9), member(2, 9), member(3, 7)];
    sort_and_deduplicate(&mut facts);
    assert_eq!(facts, vec![member(1, 9), member(2, 9), member(3, 7)]);
}

#[test]
fn distinct_group_reads_and_name_history_never_collapse_into_each_other() {
    let mut facts = vec![group_read(2), member(1, 9), group_read(1), group_read(2)];
    sort_and_deduplicate(&mut facts);
    assert_eq!(facts.len(), 3, "{facts:?}");
    assert!(facts.contains(&group_read(1)) && facts.contains(&group_read(2)));
    assert!(facts.contains(&member(1, 9)));
}
