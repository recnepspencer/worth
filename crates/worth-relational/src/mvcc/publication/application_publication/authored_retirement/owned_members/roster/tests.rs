//! Corrupt native observations and selectors reach the Native refusal owner.
use super::*;
use crate::{
    identity::data::PartitionId, symbols::data::ClientKey, transactions::data::CreatedEntityRef,
};
fn id(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}
fn denied(result: Result<(), TransactionCommitError>, cause: Cause, record: EntityId) {
    let Err(TransactionCommitError::Publication { error, .. }) = result else {
        panic!("{result:?}");
    };
    assert_eq!(error.authored_retirement_refusal(), Some(cause));
    assert_eq!(
        error.context.affected_records,
        vec![RecordRef::Entity(record)]
    );
}
#[test]
fn a_dangling_owned_endpoint_refuses_at_the_native_roster_owner() {
    for observed in [None, Some(KindId(2))] {
        let mut roster = OwnedMemberRoster::new(id(1));
        roster
            .observe_member(id(2), Some(KindId(1)), KindId(1))
            .unwrap();
        denied(
            roster.observe_member(id(3), observed, KindId(1)),
            Cause::DanglingOwnedMember,
            id(3),
        );
    }
}
#[test]
fn an_unborn_extra_member_refuses_at_the_native_roster_owner() {
    let mut roster = OwnedMemberRoster::new(id(1));
    roster
        .observe_member(id(2), Some(KindId(1)), KindId(1))
        .unwrap();
    let mut role = RequiredOutputRoleTarget {
        role: "retired-member.first".into(),
        entity_name: "Member".into(),
        kind: KindId(1),
        posture: RequiredOutputRolePosture::Retire,
        target: EntityReference::Existing(id(2)),
    };
    roster.supply(&role).unwrap();
    role.target = EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw("unborn"),
    });
    denied(roster.supply(&role), Cause::ExtraOwnedMember, id(1));
}

#[test]
fn an_owner_adjacency_entry_without_metadata_refuses_with_the_relation_subject() {
    let id = crate::identity::data::RelationId::new(PartitionId::main(), 3, 1);
    let Err(TransactionCommitError::Publication { error, .. }) = owner_metadata(id, None) else {
        panic!("missing owner metadata admitted");
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(Cause::DanglingOwnedMember)
    );
    assert_eq!(
        error.context.affected_records,
        vec![RecordRef::Relation(id)]
    );
}
