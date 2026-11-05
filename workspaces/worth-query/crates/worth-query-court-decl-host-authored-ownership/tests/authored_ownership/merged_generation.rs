//! A real merge with an older branch cannot revive a superseded ownership generation.
use super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
    lifecycle::*,
    *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    publication::AuthoredRetirementRefusal,
    transactions::{RecordRef, TransactionCommitError},
};
#[test]
fn a_fence_after_merge_names_the_current_publication_generation() {
    type S = Model<6>;
    let host = install_definition::<S, TwoProgram>();
    let runtime = host.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let created = create(&host, &request, 10, vec![20]);
    let owner = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    settle(&host, &request, 10);
    let side = runtime
        .with_host_advancement(|phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            runtime
                .branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
                .into_owned_head()
        })
        .unwrap();
    let other_request = runtime.request(&principal, &scope).on_branch(side.branch());
    create(&host, &other_request, 40, vec![]);
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            scope: 1,
            id: 20,
            co_anchors: vec![],
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"current-before-merge".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    let current = receipt.commit_reference().commit_id;
    let (merged, result) = runtime
        .probe_current_ownership_after_merge_for_test(
            runtime.current_world(),
            side.branch(),
            owner,
            &scope,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        merged.parents.len(),
        2,
        "the real merge published two parents"
    );
    assert!(merged.parents.contains(&current));
    let error =
        result.expect_err("external owned deletion must still refuse after the actual merge");
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
            publication: current,
            owner
        })
    );
    assert_eq!(
        error.context.affected_records.last(),
        Some(&RecordRef::Entity(owner))
    );
    for id in [10, 101, 103] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    for id in [20, 40, 102] {
        assert!(request.query(query::Input { id }).execute().is_err());
    }
    assert!(other_request
        .query(query::Input { id: 20 })
        .execute()
        .is_ok());
}
