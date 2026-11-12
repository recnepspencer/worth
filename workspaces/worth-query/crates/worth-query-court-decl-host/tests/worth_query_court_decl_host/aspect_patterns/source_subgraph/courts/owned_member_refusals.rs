//! Corruption twins retain their typed cause and leave the committed graph intact.
use super::super::roster::*;
use super::owned_members::{created, image};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
mod retirement;
use retirement::{command, denied, retire};
#[test]
fn a_foreign_member_and_a_repeated_member_have_distinct_native_refusals() {
    for repeated in [false, true] {
        let host = world::install(world::Fault::None, Hashes::default());
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        created(&host, &request, image(3));
        let mut other = image(1);
        other.key = "other-roster".into();
        other.members[0].key = "foreign-part".into();
        created(&host, &request, other);
        let before = request.retain_current_read().unwrap();
        let mut input = command(3);
        input.members.push(if repeated {
            "part-0".into()
        } else {
            "foreign-part".into()
        });
        let denial = denied(retire(&host, &request, input));
        let error = denial
            .authored_retirement_error()
            .unwrap_or_else(|| panic!("{denial:?}"));
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(if repeated {
                AuthoredRetirementRefusal::RepeatedOwnedMember
            } else {
                AuthoredRetirementRefusal::ForeignOwnedMember
            })
        );
        assert_eq!(error.context.affected_records.len(), 1);
        assert_eq!(
            before.selected_commit(),
            request.retain_current_read().unwrap().selected_commit()
        );
        assert_eq!(
            request
                .query(query::Read("roster".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .members
                .len(),
            3
        );
    }
}
#[test]
fn an_outside_incoming_relation_keeps_the_original_endpoint_deletion_denial() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, image(3));
    let mut other = image(1);
    other.key = "other-roster".into();
    other.members[0].key = "foreign-part".into();
    created(&host, &request, other);
    let linked = request
        .advanced()
        .mutate(links::Connect {
            from: "foreign-part".into(),
            to: "part-0".into(),
        })
        .without_source()
        .idempotency(&"outside-link".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            linked,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{linked:?}"
    );
    let before = request.retain_current_read().unwrap();
    let denial = denied(retire(&host, &request, command(3)));
    assert!(
        matches!(
            denial.relation_law(),
            Some(WorthQueryRelationLawDenial::EndpointDeletion { .. })
        ),
        "{denial:?}"
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
}
#[test]
fn a_roster_preimage_over_the_shared_byte_bound_refuses_before_effects() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, image(20));
    let before = request.retain_current_read().unwrap();
    let denial = denied(retire(&host, &request, command(20)));
    let failure = denial
        .relation_preimage_failure()
        .unwrap_or_else(|| panic!("{denial:?}"));
    assert_eq!(
        failure.kind(),
        WorthQueryRelationPreImageDenial::ExceedsByteBound
    );
    assert!(!failure.affected_records().is_empty());
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert_eq!(
        request
            .query(query::Read("roster".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .members
            .len(),
        20
    );
}

#[test]
fn member_fields_alone_can_exceed_the_roster_preimage_bound_before_effects() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut graph = image(10);
    let keys = (0..10)
        .map(|index| format!("part-{index}-{}", "k".repeat(120)))
        .collect::<Vec<_>>();
    for (index, row) in graph.members.iter_mut().enumerate() {
        row.key = keys[index].clone();
        row.next = keys.get(index + 1).cloned();
    }
    let input = Retire {
        key: graph.key.clone(),
        members: keys,
        replacement: None,
    };
    created(&host, &request, graph);
    let before = request.retain_current_read().unwrap();
    let denial = denied(retire(&host, &request, input));
    assert_eq!(
        denial.preimage_retention_denial(),
        Some(WorthQueryPreImageRetentionDenial::ExceedsByteBound)
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
}

#[test]
fn a_member_cannot_retire_through_its_family_while_the_anchor_stays() {
    let host = world::install(world::Fault::RetainAuthoredAnchor, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, image(1));
    let before = request.retain_current_read().unwrap();
    let result = request
        .advanced()
        .mutate(command(1))
        .without_source()
        .idempotency(&"member-alone".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation);
    let Err(WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    )) = result
    else {
        panic!("{result:?}");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::MissingOutputRole
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert_eq!(
        request
            .query(query::Read("roster".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .members
            .len(),
        1
    );
}
#[test]
fn a_member_cannot_restore_without_its_anchor_output() {
    let host = world::install(world::Fault::OmitRestoredAnchor, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, image(1));
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } =
        retire(&host, &request, command(1))
    else {
        panic!("retirement failed");
    };
    let before = request.retain_current_read().unwrap();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&selected)
        .idempotency("member-without-anchor")
        .limits(limits)
        .prepare();
    let Err(WorthQueryApplicationCorrectionDenial::Mutation(
        WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        ),
    )) = preparation
    else {
        panic!("unexpected preparation");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::MissingOutputRole
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    super::owned_members::assert_absent(&request);
}

#[test]
fn a_changed_member_key_leaves_the_original_inverse_incomplete() {
    refuse_member(
        world::Fault::ChangedRestoredKey,
        WorthQueryRelationPreImageDenial::IncompleteInverse,
    );
}
#[test]
fn two_fresh_members_with_the_original_key_refuse_as_repeated() {
    refuse_member(
        world::Fault::RepeatedRestoredMember,
        WorthQueryRelationPreImageDenial::RepeatedInverse,
    );
}
#[test]
fn a_restored_member_under_another_live_owner_refuses_as_foreign() {
    refuse_member(
        world::Fault::ForeignRestoredMember,
        WorthQueryRelationPreImageDenial::ForeignEndpoint,
    );
}
#[test]
fn recreating_a_member_that_was_never_deleted_refuses_as_extra() {
    refuse_member(
        world::Fault::ExtraRestoredMember,
        WorthQueryRelationPreImageDenial::ExtraInverse,
    );
}
#[test]
fn a_restored_members_fields_must_equal_the_retained_preimage() {
    refuse_member(
        world::Fault::ChangedRestoredValue,
        WorthQueryRelationPreImageDenial::RestoredFieldMismatch,
    );
}
fn refuse_member(fault: world::Fault, expected: WorthQueryRelationPreImageDenial) {
    use super::owned_members::{delete_one, local_image, local_snapshot};
    use world::Fault::ExtraRestoredMember;

    let host = world::install(fault, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, local_image());
    let original = local_snapshot(&request).members[0].0;
    let receipt = delete_one(&host, &request);
    let before = local_snapshot(&request);
    let retained = request.retain_current_read().unwrap();
    let commit = retained.selected_commit();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&selected)
        .idempotency("refuse-one-member")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = preparation else {
        panic!("inverse did not prepare");
    };
    let result = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("{fault:?}: {result:?}");
    };
    let failure = denial
        .relation_preimage_failure()
        .unwrap_or_else(|| panic!("{fault:?}: {denial:?}"));
    assert_eq!(failure.kind(), expected, "{fault:?}");
    if fault == ExtraRestoredMember {
        assert!(
            matches!(failure.affected_records(), [worth_relational::facade::transactions::RecordRef::Entity(id)] if *id != original)
        );
    } else {
        assert_eq!(
            failure.affected_records(),
            &[worth_relational::facade::transactions::RecordRef::Entity(
                original
            )]
        );
    }
    assert_eq!(
        commit,
        request.retain_current_read().unwrap().selected_commit()
    );
    assert_eq!(before, local_snapshot(&request));
}
