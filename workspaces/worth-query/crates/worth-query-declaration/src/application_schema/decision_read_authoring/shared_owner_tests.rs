//! Multiple shared-owner managed-index Parents on one governing relation share
//! one canonical governing relation decision read without `DuplicateMember`.
mod fixture;

use super::super::*;
use fixture::{add_shared, base, gov_ref, op_ref, EdgeBoxes, FaceBoxes, VertexBoxes};

#[test]
fn multiple_shared_owner_parents_on_one_relation_share_one_governing_read() {
    let built = add_shared::<FaceBoxes>(add_shared::<EdgeBoxes>(add_shared::<VertexBoxes>(base())))
        .build()
        .expect("three Parents on one governing relation must not hit DuplicateMember");
    let reads: Vec<_> =
        application_operation_decision_read_targets(built.erased().members(), "Read").collect();
    let relation_count = reads
        .iter()
        .filter(|t| {
            matches!(
                t,
                ApplicationOperationDecisionReadTarget::Relation { relation, from, to }
                    if relation == "Gov" && from == "Owner" && to == "Member"
            )
        })
        .count();
    let shared_count = reads
        .iter()
        .filter(|t| {
            matches!(
                t,
                ApplicationOperationDecisionReadTarget::SharedOwnerManagedIndex { .. }
            )
        })
        .count();
    assert_eq!(relation_count, 1);
    assert_eq!(shared_count, 3);
}

#[test]
fn explicit_relation_read_and_shared_owner_read_are_order_independent_and_canonical() {
    let single = add_shared::<VertexBoxes>(base()).build().unwrap();
    let explicit_first =
        add_shared::<VertexBoxes>(base().operation_read_relation(op_ref(), gov_ref()))
            .build()
            .expect("explicit relation read before shared-owner read");
    let explicit_after = add_shared::<VertexBoxes>(base())
        .operation_read_relation(op_ref(), gov_ref())
        .build()
        .expect("explicit relation read after shared-owner read");
    assert_eq!(explicit_first.identity(), single.identity());
    assert_eq!(explicit_after.identity(), single.identity());
    assert_eq!(explicit_first.erased().members(), single.erased().members());
    assert_eq!(explicit_after.erased().members(), single.erased().members());
}

#[test]
fn duplicate_same_parent_or_double_explicit_relation_still_refuses() {
    let dup_parent = add_shared::<VertexBoxes>(add_shared::<VertexBoxes>(base())).build();
    assert_eq!(
        dup_parent.unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
    let dup_relation = add_shared::<VertexBoxes>(
        base()
            .operation_read_relation(op_ref(), gov_ref())
            .operation_read_relation(op_ref(), gov_ref()),
    )
    .build();
    assert_eq!(
        dup_relation.unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}

#[test]
fn contributed_shared_owner_parents_assign_governing_relation_ownership_cleanly() {
    let core_id = ApplicationSchemaContributionIdentity::new("worth.query.tests.core.v1");
    let op_id = ApplicationSchemaContributionIdentity::new("worth.query.tests.op.v1");
    let peer_id = ApplicationSchemaContributionIdentity::new("worth.query.tests.peer.v1");

    // One contribution declaring three shared-owner Parents owns the single implied relation read.
    let mut single_owner = base();
    single_owner.retain_contribution_closure(core_id.clone(), 0, 0);
    let op_start = single_owner.contribution_member_count();
    let mut single_owner = add_shared::<FaceBoxes>(add_shared::<EdgeBoxes>(add_shared::<
        VertexBoxes,
    >(single_owner)));
    single_owner.retain_contribution_closure(op_id.clone(), op_start, 0);
    assert_eq!(single_owner.build().unwrap().contributions().len(), 2);

    // Explicit owner in one contribution + shared-owner read in another composes without overlap.
    let mut split_explicit = base();
    split_explicit.retain_contribution_closure(core_id.clone(), 0, 0);
    let op_start = split_explicit.contribution_member_count();
    let mut split_explicit = split_explicit.operation_read_relation(op_ref(), gov_ref());
    split_explicit.retain_contribution_closure(op_id.clone(), op_start, 0);
    let peer_start = split_explicit.contribution_member_count();
    let mut split_explicit = add_shared::<VertexBoxes>(split_explicit);
    split_explicit.retain_contribution_closure(peer_id.clone(), peer_start, 0);
    assert_eq!(split_explicit.build().unwrap().contributions().len(), 3);

    // Two contributions implying the same unowned governing relation refuse cleanly as overlap.
    let mut split_overlap = base();
    split_overlap.retain_contribution_closure(core_id, 0, 0);
    let op_start = split_overlap.contribution_member_count();
    let mut split_overlap = add_shared::<VertexBoxes>(split_overlap);
    split_overlap.retain_contribution_closure(op_id, op_start, 0);
    let peer_start = split_overlap.contribution_member_count();
    let mut split_overlap = add_shared::<EdgeBoxes>(split_overlap);
    split_overlap.retain_contribution_closure(peer_id, peer_start, 0);
    assert_eq!(
        split_overlap.build().unwrap_err(),
        ApplicationSchemaDeclarationDenial::OverlappingContributionClosure
    );
}
