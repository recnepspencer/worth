//! Corruption twins retain their typed cause and leave the committed graph intact.
use super::super::roster::*;
use super::owned_members::{created, image};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
fn retire(
    host: &world::Host,
    request: &world::Request<'_, '_, '_>,
    input: Retire,
) -> WorthQueryApplicationMutationOutcome<edit::Denial, edit::Edited> {
    request
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&"refuse-roster".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}
fn denied(
    outcome: WorthQueryApplicationMutationOutcome<edit::Denial, edit::Edited>,
) -> WorthQueryApplicationCommitDenial {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}");
    };
    denial
}
fn command(count: usize) -> Retire {
    Retire {
        key: "roster".into(),
        members: image(count)
            .members
            .into_iter()
            .map(|row| row.key)
            .collect(),
        replacement: None,
    }
}
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
fn a_dangling_intra_roster_endpoint_is_a_typed_domain_refusal_before_effects() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = request.retain_current_read().unwrap();
    let mut graph = image(3);
    graph.members[1].next = Some("missing-part".into());
    let outcome = request
        .advanced()
        .mutate(Create(graph))
        .without_source()
        .idempotency(&"dangling-link".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(edit::Denial::InvalidSourceGraph)
        ),
        "{outcome:?}"
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert!(request
        .query(query::Read("roster".into()))
        .execute()
        .is_err());
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
fn a_created_extra_member_cannot_enter_an_authored_retirement_roster() {
    let host = world::install(world::Fault::ExtraAuthoredRetireMember, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, image(3));
    let before = request.retain_current_read().unwrap();
    let error = request
        .advanced()
        .mutate(command(3))
        .without_source()
        .idempotency(&"extra-created-member".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(error),
    ) = error
    else {
        panic!("{error:?}");
    };
    let cause = error
        .downcast::<WorthQueryApplicationAttemptDenial>()
        .unwrap();
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
}
#[test]
fn member_fields_alone_can_exceed_the_roster_preimage_bound_before_effects() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, image(100));
    let before = request.retain_current_read().unwrap();
    let denial = denied(retire(&host, &request, command(100)));
    assert_eq!(
        denial.preimage_retention_denial(),
        Some(WorthQueryPreImageRetentionDenial::ExceedsByteBound)
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
}
