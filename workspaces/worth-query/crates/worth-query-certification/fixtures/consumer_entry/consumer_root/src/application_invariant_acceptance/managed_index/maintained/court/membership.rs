//! A real link creates the only violating Source membership without a public B handle.
use super::super::authoring::Edit;
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;

#[test]
fn authored_membership_into_unopened_scope_runs_its_rule_and_refuses_only_the_conflicting_link() {
    lifecycle::run(journey);
}
fn journey() {
    let provider = Arc::new(prefix_observation::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
    let world = host::install(Arc::clone(&provider), Arc::clone(&rules));
    // Eligibility is compiled from these actual catalog entries. The prefix
    // and box Parents use different ordering from Source; their output routes
    // still carry every membership-preserving projection/order influence.
    let schema = world.installed_schema();
    let prefix = schema
        .installed_query_binding::<query::PrefixBinding>()
        .unwrap();
    let prefix = schema
        .installed_managed_index_parent(&prefix, &mut |_, _| Ok::<_, ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(prefix.access_contract().meaning().predicate_profile(), Ok(worth_query_host::facade::domain::WorthQueryManagedIndexPredicateProfile::ConservativeProjectedRefinement));
    let boxes = schema
        .installed_query_binding::<super::super::geometry::query::BoxBinding>()
        .unwrap();
    let boxes = schema
        .installed_managed_index_parent(&boxes, &mut |_, _| Ok::<_, ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(boxes.access_contract().meaning().predicate_profile(), Ok(worth_query_host::facade::domain::WorthQueryManagedIndexPredicateProfile::DirectSourceProjection));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    for (key, edit) in [
        Edit::Directory {
            identity: "C".into(),
        },
        Edit::Item {
            directory: "A".into(),
            identity: "z".into(),
            name: "taken".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let result = request
            .advanced()
            .mutate(edit)
            .idempotency(&(0x919310 + key as u64))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{result:?}"
        );
    }
    rules.records.lock().unwrap().clear();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let result = request
        .advanced()
        .mutate(Edit::Share {
            directory: "B".into(),
            item: "z".into(),
        })
        .idempotency(&0x919312u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    require_violation(result);
    {
        let records = rules.records.lock().unwrap();
        let b = records
            .iter()
            .find(|record| record.source.after.len() == 2)
            .expect("B actually observes the proposed y/z conflict");
        assert_eq!(
            values(b.source.before.as_ref().unwrap()),
            vec![("y".into(), "taken".into())]
        );
        assert_eq!(
            values(&b.source.after),
            vec![("y".into(), "taken".into()), ("z".into(), "taken".into())]
        );
    }
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    rules.records.lock().unwrap().clear();
    let result = request
        .advanced()
        .mutate(Edit::Share {
            directory: "C".into(),
            item: "z".into(),
        })
        .idempotency(&0x919313u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    {
        let records = rules.records.lock().unwrap();
        let c = records
            .iter()
            .find(|record| {
                record.source.before.as_ref().is_some_and(Vec::is_empty)
                    && record.source.after.len() == 1
            })
            .expect("the empty, unopened C Source must run its actual rule on the new membership");
        assert_eq!(values(&c.source.after), vec![("z".into(), "taken".into())]);
    }
    let b = request
        .query(find("B", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let c = request
        .query(find("C", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(
        values(request.query(find("B", "")).execute().unwrap().rows()),
        expected("B", "shared", "")
    );
    assert_eq!(
        values(request.query(find("C", "")).execute().unwrap().rows()),
        vec![("z".into(), "taken".into())]
    );
    b.close();
    c.close();
}
