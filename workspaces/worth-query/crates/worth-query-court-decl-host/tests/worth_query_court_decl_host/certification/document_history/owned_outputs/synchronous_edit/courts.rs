//! Ordinary source edits and genuine Preserve writers share the final publication.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationReadObservation as ReadObservation,
    runtime::ExecutionAllocationPolicy,
};

pub(super) fn commit_ordinal(host: &world::Host) -> u64 {
    host.with_host_advancement(|phase| {
        ReadObservation::retain_on_branch(&phase, host.runtime(), host.current_world())
            .unwrap()
            .selected_commit()
            .ordinal()
    })
    .unwrap()
}

pub(super) fn native_head<P: ApplicationProgramDefinition<Schema>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
) -> (u64, Vec<u64>) {
    host.with_host_advancement(|phase| {
        let retained =
            ReadObservation::retain_on_branch(&phase, host.runtime(), host.current_world())
                .unwrap();
        let selected = retained.select_on(&phase, host.runtime()).unwrap();
        let target = selected
            .product()
            .relational_basis_descriptor()
            .reference()
            .target()
            .as_basis()
            .unwrap();
        (
            target.selected_commit_id(),
            target.parent_commit_ids().to_vec(),
        )
    })
    .unwrap()
}

pub(super) fn edit_two_families(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(host, &request, "a");
    second::settle(host, &request, "a");
    let original = world::outputs(host, host.current_world());
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let before = commit_ordinal(host);
    let native_before = native_head(host);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"synchronous-edit".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(world::model(host, host.current_world()).value, 2);
    let outputs = world::outputs(host, host.current_world());
    assert_eq!((outputs.0.value, outputs.1.value), (20, 1002));
    assert_eq!(
        (outputs.0.native_id, outputs.1.native_id),
        (original.0.native_id, original.1.native_id)
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let receipt = outcome.receipt().unwrap();
    let native_after = native_head(host);
    assert_eq!(
        native_after.0,
        receipt.commit_reference().commit_id.0,
        "the edit receipt is the final Native commit"
    );
    assert_eq!(
        native_after.1,
        [native_before.0],
        "the edit contributes one actual Native ancestry step"
    );
    assert_eq!(
        receipt.commit_reference().parents,
        [worth_relational::facade::history::CommitId(native_before.0)]
    );
    for id in [
        world::model(host, host.current_world()).native_id,
        outputs.0.native_id,
        outputs.1.native_id,
    ] {
        let changes = receipt
            .committed_changes()
            .entity_changes()
            .filter(|(changed, _)| *changed == id)
            .collect::<Vec<_>>();
        assert_eq!(
            changes,
            [(
                id,
                worth_relational::facade::publication::RecordStructuralChange::Updated
            )],
            "one receipt carries this source or producer's actual change"
        );
    }
    assert_eq!(
        commit_ordinal(host),
        before + 1,
        "one composite publication for the complete edit"
    );
    first::settle(host, &request, "a");
    second::settle(host, &request, "a");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "the coupled results settle without republishing"
    );
    assert_eq!(
        commit_ordinal(host),
        before + 1,
        "readiness and measure reads commit nothing"
    );
    assert_eq!(
        native_head(host),
        native_after,
        "readiness publishes no Native metadata commit either"
    );
    let replay = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"synchronous-edit".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed) = replay else {
        panic!("{replay:?}")
    };
    assert_eq!(
        replayed.commit_reference(),
        outcome.receipt().unwrap().commit_reference()
    );
    assert_eq!(
        commit_ordinal(host),
        before + 1,
        "the whole edit is one retry unit"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    assert_eq!(native_head(host), native_after);
}
#[test]
fn ordinary_edit_and_two_preserve_writers_publish_together() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    edit_two_families(&world::install());
}
#[test]
fn worker_only_ordinary_edit_and_two_preserve_writers_publish_together() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    edit_two_families(&world::install_worker_only());
}
