//! A genuine prefix Source from the sibling cannot authorize this fork's mutation.
use super::super::super::super::{authoring as catalog, mutation, query as names};
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use worth_query_host::facade::product::WorthQueryProductBranch;

pub(super) fn seed(world: &host::Host, principal: &Principal, scope: &WorthQueryRequestScope) {
    let result = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(catalog::Edit::Item {
            directory: "A".into(),
            identity: "fork-name".into(),
            name: "original".into(),
        })
        .idempotency(&0x919432u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
fn find() -> names::FindPrefix {
    names::FindPrefix {
        directory: "A".into(),
        prefix: "".into(),
    }
}
fn rename() -> mutation::Rename {
    mutation::Rename {
        directory: "A".into(),
        prefix: "".into(),
        item: "fork-name".into(),
        replacement: "updated".into(),
    }
}
pub(super) fn exercise(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    calls: &prefix_observation::Calls,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let sibling = world.runtime().request(principal, scope).on_branch(branch);
    let source = request
        .query(find())
        .execute()
        .unwrap()
        .into_result_set_observation()
        .unwrap();
    let before = (calls.totals(), calls.updates.load(Ordering::SeqCst));
    let denied = sibling
        .advanced()
        .mutate(rename())
        .expect_result_set(source.clone())
        .idempotency(&0x919433u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::SourceExpectation(denial) = denied else {
        panic!("the actual sibling Source must refuse its branch association: {denied:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQuerySourceExpectationDenialKind::ForeignBranch
    );
    assert_eq!(
        (calls.totals(), calls.updates.load(Ordering::SeqCst)),
        before,
        "foreign Source refuses before prefix provider callbacks"
    );
    let valid = request
        .advanced()
        .mutate(rename())
        .expect_result_set(source)
        .idempotency(&0x919433u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            valid,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{valid:?}"
    );
    assert_eq!(
        request.query(find()).execute().unwrap().rows()[0].name,
        "updated"
    );
    assert_eq!(
        sibling.query(find()).execute().unwrap().rows()[0].name,
        "original"
    );
}
