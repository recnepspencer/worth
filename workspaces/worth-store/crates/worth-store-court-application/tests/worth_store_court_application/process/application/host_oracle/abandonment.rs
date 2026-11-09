use super::*;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::WorthQueryApplicationUncommitted, WorthQueryApplicationHomeLocation,
};

#[test]
fn caller_abandonment_uses_the_actual_typed_outcome_and_retained_owner_effect() {
    let path = tempfile::tempdir().unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for location in [
        WorthQueryApplicationHomeLocation::MemoryOnly,
        WorthQueryApplicationHomeLocation::Directory(path.path().to_path_buf()),
    ] {
        let directory = matches!(&location, WorthQueryApplicationHomeLocation::Directory(_));
        let home = host_observation::opening::open(
            location,
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            None,
            Default::default(),
            &execution,
        );
        let before = host_observation::projection::observe(home.runtime(), &execution);
        let published_balance = host_observation::requests::balance(home.runtime());
        assert_ne!(published_balance, 90_991);
        let outcome =
            host_interruption::publication::abandon_after_owner(home.runtime(), 192, 90_991);
        if directory {
            let WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::ProductPreparationPending(pending),
            ) = &outcome
            else {
                panic!("Home retains its original preparation: {outcome:?}")
            };
            assert_eq!(pending.stage(), worth_query_host::facade::primary_graph::WorthQueryProductPreparationStage::Publishing);
        } else {
            assert!(
                matches!(
                    outcome,
                    WorthQueryApplicationMutationOutcome::Commit(
                        WorthQueryApplicationUncommitted::Indeterminate(_)
                    )
                ),
                "Memory isolates the actual provider unwind: {outcome:?}"
            );
        }
        let projected = host_observation::receipts::observe(&outcome, &before.head);
        assert_eq!(
            projected.fate,
            if directory {
                host_observation::receipts::MutationFate::ProductPreparationPending
            } else {
                host_observation::receipts::MutationFate::Unresolved
            }
        );
        assert!(
            projected.receipt.is_none(),
            "no product receipt was returned"
        );
        let original = if let WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::ProductPreparationPending(pending),
        ) = outcome
        {
            Some(pending.try_into_recovery().unwrap_or_else(|pending| {
                panic!("the same original preparation must hand off: {pending:?}")
            }))
        } else {
            None
        };
        let after = host_observation::projection::observe(home.runtime(), &execution);
        assert_eq!(after.head.commit, before.head.commit);
        assert_eq!(after.head.generation, before.head.generation);
        assert_eq!(
            host_observation::requests::balance(home.runtime()),
            published_balance,
            "queries retain the unmoved product basis"
        );
        let account_kind =
            <worth_query_testkit_host::facade::host::application_home::schema::Account as worth_query_host::facade::declaration::application_schema::fragment::Entity>::NAME;
        let account = after
            .entities
            .iter()
            .find(|e| e.kind_name == account_kind)
            .expect("the actual account row");
        let changed_balance =
            serde_json::to_value(worth_foundational::facade::AspectValue::UInt64(90_991)).unwrap();
        assert!(
            account
                .authoritative_aspect_state
                .as_object()
                .unwrap()
                .values()
                .any(|aspect| {
                    aspect["value"]["Struct"].as_object().is_some_and(|fields| {
                        fields.values().any(|value| value == &changed_balance)
                    })
                }),
            "the actual Native account retains the performed balance: {account:?}"
        );
        let [custody] = after.custody.as_slice() else {
            panic!(
                "one actual abandoned owner effect must remain: {:?}",
                after.custody
            )
        };
        assert!(matches!(custody.effects, Some(count) if count > 0));
        if let Some(original) = original {
            use worth_query_host::facade::primary_graph::{
                WorthQueryProductUnpublishedRecoveryReleaseDenial as Denial,
                WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure,
            };
            let failure = home
                .runtime()
                .runtime()
                .release_product_publication_recovery(original, 0)
                .expect_err("a persistent live open requires reopen");
            let Failure::Recovery(failure) = failure else {
                panic!("no owner cleanup was authorized")
            };
            assert_eq!(
                failure.denial(),
                Denial::World(
                    worth_runtime_world::facade::RuntimeWorldRecoveryDenial::ReopenRequired
                )
            );
            let retained = failure.into_recovery();
            assert_eq!(
                retained.record_handle().identity().ordinal(),
                custody.identity,
                "the exact original is returned; it was not released"
            );
        } else {
            assert_eq!(
                host_interruption::publication::release_custody(home.runtime()).unwrap(),
                1
            );
        }
    }
}
