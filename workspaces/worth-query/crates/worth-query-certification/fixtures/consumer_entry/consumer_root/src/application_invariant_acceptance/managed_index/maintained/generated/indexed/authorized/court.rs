//! Independent Source READ denial crosses the real synchronous producer entrance.
use super::program;
use crate::application_invariant_acceptance::{
    authentication,
    managed_index::{lifecycle, maintained as catalog},
};
use catalog::{
    generated::{indexed::request, source},
    host, query, source_authorization as protected_source,
};
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

#[test]
fn independently_denied_applied_source_read_discards_the_authored_candidate() {
    lifecycle::run(|| {
        let calls = protected_source::Configuration::default();
        let world = host::install_source_authorization(calls.clone());
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let current = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let seeded = current
            .advanced()
            .mutate(catalog::authoring::Edit::Item {
                directory: "catalog".into(),
                identity: "authorized-item".into(),
                name: "shared".into(),
            })
            .idempotency(&0x919d_000fu64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            seeded,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let original = current
            .query(protected_source::FindNative {
                directory: "catalog".into(),
                prefix: "sh".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(original.rows().len(), 1);
        assert_eq!(
            (&*original.rows()[0].identity, &*original.rows()[0].name),
            ("authorized-item", "shared")
        );
        let authored_before = current
            .query(source::ReadDirectory {
                directory: "catalog".into(),
            })
            .execute()
            .unwrap();
        let change_read = |readable, key| {
            let outcome = current
                .advanced()
                .mutate(catalog::protected::status::SetStatus {
                    identity: "watch-access".into(),
                    active: true,
                    readable,
                })
                .without_source()
                .idempotency(&key)
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
        };
        change_read(false, 0x919d_0010u64);
        let head = current
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let callbacks = (
            calls.native.probe_validations.load(Ordering::SeqCst),
            calls.native.matched_sources.load(Ordering::SeqCst),
        );
        // RequestItem has Public operation admission. Only its required producer's
        // independently admitted Source READ is now forbidden.
        let outcome = current
            .advanced()
            .mutate(request::Request(source::operation::RequestItem {
                directory: "catalog".into(),
                name: "denied-applied-source".into(),
            }))
            .without_source()
            .idempotency(&0x919d_0011u64)
            .execute_performed::<host::Program, program::Root>(
                &world,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
                denial,
            )),
        ) = outcome
        else {
            panic!("the independently denied Source READ must refuse the candidate")
        };
        let producer = denial
            .coupled_producer_denial()
            .expect("actual required Source discovery refusal");
        let source_denial = producer
            .managed_source_query_denial()
            .unwrap_or_else(|| panic!(
                "original managed Source admission cause; actual producer={:?} stage={:?} detail={:?}",
                producer.kind(), denial.stage(), denial.detail()
            ));
        let expected = WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied,
            ),
        );
        assert!(
            matches!(source_denial.cause(), WorthQueryManagedIndexDenialCause::QueryAdmission(cause) if cause.kind() == expected)
        );
        assert_eq!(
            producer.kind(),
            WorthQueryOutputDemandDenialKind::SourceQueryAdmission(expected)
        );
        assert_eq!(
            (
                calls.native.probe_validations.load(Ordering::SeqCst),
                calls.native.matched_sources.load(Ordering::SeqCst)
            ),
            callbacks,
            "Source admission refuses before managed provider work"
        );
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &head
        );
        assert_eq!(
            current
                .query(source::ReadDirectory {
                    directory: "catalog".into()
                })
                .execute()
                .unwrap()
                .rows(),
            authored_before.rows()
        );
        assert!(current
            .query(query::FindPrefix {
                directory: "catalog".into(),
                prefix: "produced:".into()
            })
            .execute()
            .unwrap()
            .rows()
            .is_empty());
        drop(denial);
        change_read(true, 0x919d_0012u64);
        assert_eq!(
            current
                .query(protected_source::FindNative {
                    directory: "catalog".into(),
                    prefix: "sh".into()
                })
                .execute()
                .unwrap()
                .rows(),
            original.rows()
        );
        // A fresh request with restored READ reaches the exact same installed
        // producer. This rules out a permanently broken output/source setup.
        let fresh_scope = authentication::request_scope();
        let fresh = world
            .runtime()
            .request(&principal, &fresh_scope)
            .on_branch(world.current_world());
        let outcome = fresh
            .advanced()
            .mutate(request::Request(source::operation::RequestItem {
                directory: "catalog".into(),
                name: "allowed-applied-source".into(),
            }))
            .without_source()
            .idempotency(&0x919d_0013u64)
            .execute_performed::<host::Program, program::Root>(
                &world,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
            panic!("restored independent READ reaches the genuine required producer")
        };
        let publication = performed
            .receipt()
            .committed_product_publication()
            .composite_commit()
            .clone();
        let produced = fresh
            .query(query::FindPrefix {
                directory: "catalog".into(),
                prefix: "produced:".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(produced.rows().len(), 1);
        assert_eq!(
            (&*produced.rows()[0].identity, &*produced.rows()[0].name),
            ("managed-authorized-item-shared", "produced:shared")
        );
        // This protected Parent is current-only. Admit its real fresh Current
        // demand; performed.start_required_outputs deliberately selects a pinned
        // original Source, which this Parent does not declare. Keep the actual
        // co-commit receipt while verifying retained output reuse independently.
        let mut output = fresh
            .demand(super::Demand {
                directory: "catalog".into(),
            })
            .start_in_program::<host::Program, program::Root>(&world)
            .expect("restored current-only Source admits fresh delivery");
        let settlement = (0..64)
            .find_map(|_| match output.advance(&fresh).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                _ => None,
            })
            .expect("restored required output settles");
        assert_eq!(settlement.producer_contacts_in_this_demand(), 0);
        assert_eq!(
            fresh.retain_current_read().unwrap().selected_commit(),
            &publication,
            "fresh delivery made no new publication"
        );
        assert_eq!(
            performed
                .receipt()
                .committed_product_publication()
                .composite_commit(),
            &publication
        );
        let correspondence = settlement
            .outputs_of::<catalog::generated::operation::Outputs>()
            .unwrap();
        let entity = correspondence
            .entity::<catalog::generated::operation::ItemOutput>()
            .unwrap()
            .entity_id();
        assert_eq!(
            (
                entity.partition_value(),
                entity.local_slot_value(),
                entity.generation_value()
            ),
            produced.rows()[0].coordinate
        );
        let repeated = (0..64)
            .find_map(|_| match output.advance(&fresh).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                _ => None,
            })
            .expect("restored current-only delivery stays settled");
        assert_eq!(repeated.producer_contacts_in_this_demand(), 0);
        assert_eq!(
            fresh.retain_current_read().unwrap().selected_commit(),
            &publication
        );
        drop((output, settlement, repeated, performed));
        // Real branch close refuses leaked participant/recovery custody.
        let closed = world
            .runtime()
            .with_host_advancement(|phase| {
                world
                    .runtime()
                    .on_branch(world.current_world())
                    .close(&phase)
                    .unwrap()
            })
            .unwrap();
        assert!(
            closed.is_complete(),
            "Source refusal and restored output leave closable custody"
        );
    });
}
