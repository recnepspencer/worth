//! Round trips compare literal graph meaning by keys; ids prove only fresh allocation.
use super::super::roster::*;
use super::*;
use worth_relational::facade::transactions::RecordRef;
pub(super) fn image(count: usize) -> Image {
    Image {
        key: "roster".into(),
        value: 7,
        members: (0..count)
            .map(|i| Row {
                key: format!("part-{i}"),
                value: 20 + i as u64,
                next: (i + 1 < count).then(|| format!("part-{}", i + 1)),
            })
            .collect(),
    }
}
pub(super) fn created(
    host: &world::Host,
    request: &world::Request<'_, '_, '_>,
    image: Image,
) -> WorthQueryApplicationCommitReceipt {
    let creation_key = format!("create:{}", image.key);
    let result = request
        .advanced()
        .mutate(Create(image))
        .without_source()
        .idempotency(&creation_key)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
        panic!("{result:?}");
    };
    receipt
}
pub(super) fn reverse(
    request: &world::Request<'_, '_, '_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    key: &str,
) {
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = preparation else {
        panic!("inverse did not prepare");
    };
    let result = prepared.publish().unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
#[test]
fn retirement_restores_a_variable_keyed_chain_and_an_empty_roster() {
    for count in [0, 3] {
        let host = world::install(world::Fault::None, Hashes::default());
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let expected = image(count);
        created(&host, &request, expected.clone());
        let before = request
            .query(query::Read("roster".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let result = request
            .advanced()
            .mutate(Retire {
                key: "roster".into(),
                members: expected.members.iter().map(|row| row.key.clone()).collect(),
                replacement: None,
            })
            .without_source()
            .idempotency(&"roster-retire".to_owned())
            .execute_in_program(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
            panic!("{result:?}");
        };
        assert_absent(&request);
        reverse(&request, &receipt, "roster-restore");
        let after = request
            .query(query::Read("roster".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!((&after.key, after.value), (&expected.key, expected.value));
        assert_eq!(
            after.members.iter().map(|(_, row)| row).collect::<Vec<_>>(),
            expected.members.iter().collect::<Vec<_>>()
        );
        assert_ne!(before.id, after.id);
        for ((old, before), (fresh, after)) in before.members.iter().zip(&after.members) {
            assert_eq!(before.key, after.key);
            assert_ne!(old, fresh);
        }
    }
}
#[test]
fn native_owner_adjacency_refuses_a_missing_member_before_any_effect() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, image(3));
    let before = request.retain_current_read().unwrap();
    let missing = request
        .query(query::Read("roster".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .iter()
        .find(|(_, row)| row.key == "part-2")
        .unwrap()
        .0;
    let result = request
        .advanced()
        .mutate(Retire {
            key: "roster".into(),
            members: vec!["part-0".into(), "part-1".into()],
            replacement: None,
        })
        .without_source()
        .idempotency(&"missing-member".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let result = result.unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("{result:?}");
    };
    let error = denial
        .authored_retirement_error()
        .unwrap_or_else(|| panic!("{denial:?}"));
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::MissingOwnedMember)
    );
    assert_eq!(
        error.context.affected_records,
        vec![RecordRef::Entity(missing)]
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
}

#[test]
fn replace_inverse_restores_the_removed_member_and_prior_link_by_key() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let expected = image(3);
    created(&host, &request, expected.clone());
    let before = request
        .query(query::Read("roster".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let mut replacement = image(2);
    replacement.value = 90;
    replacement.members[0].next = None;
    replacement.members[1].next = Some("part-0".into());
    replacement.members[0].value = 44;
    let result = request
        .advanced()
        .mutate(Retire {
            key: expected.key.clone(),
            members: expected.members.iter().map(|row| row.key.clone()).collect(),
            replacement: Some(replacement.clone()),
        })
        .without_source()
        .idempotency(&"replace-roster".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
        panic!("{result:?}");
    };
    let replaced = request
        .query(query::Read("roster".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(
        replaced
            .members
            .iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>(),
        replacement.members.iter().collect::<Vec<_>>()
    );
    reverse(&request, &receipt, "undo-replace-roster");
    let after = request
        .query(query::Read("roster".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!((&after.key, after.value), (&expected.key, expected.value));
    assert_eq!(
        after.members.iter().map(|(_, row)| row).collect::<Vec<_>>(),
        expected.members.iter().collect::<Vec<_>>()
    );
    assert_ne!(after.id, before.id);
    for ((old, prior), (fresh, restored)) in before.members.iter().zip(&after.members) {
        assert_eq!(prior.key, restored.key);
        assert_ne!(old, fresh);
    }
}

#[test]
fn the_create_inverse_retires_every_owned_member_with_its_anchor() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let receipt = created(&host, &request, image(3));
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
    reverse(&request, &receipt, "undo-create-roster");
    assert_absent(&request);
}

pub(super) fn assert_absent(request: &world::Request<'_, '_, '_>) {
    let Err(denial) = request.query(query::Read("roster".into())).execute() else {
        panic!("retired roster remains visible");
    };
    let WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial) = denial else {
        panic!("{denial:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryEntityResolutionDenialKind::Refused(
            WorthQueryEntityResolutionRefusalKind::UnknownEntity
        )
    );
}
