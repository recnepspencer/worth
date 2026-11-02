//! Actual Source content changes can preserve canonical input and complete selection.
use super::*;

#[test]
fn equal_canonical_input_reuses_managed_output_after_real_source_content_changes() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
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
        let initial = settle(&world, &request, "A");
        assert_eq!(
            initial.settlement().root_producer_contacts_in_this_demand(),
            1
        );
        let original = initial
            .read_current(&request, find("A", "produced:"))
            .unwrap()
            .rows()[0]
            .clone();
        let source = request.query(find("A", "sh")).execute().unwrap();
        let coordinate = source.rows()[0].coordinate;
        let outcome = request
            .advanced()
            .mutate(super::super::super::super::mutation::Rename {
                directory: "A".into(),
                prefix: "sh".into(),
                item: "x".into(),
                replacement: "shARED".into(),
            })
            .expect_result_set(source.into_result_set_observation().unwrap())
            .idempotency(&0x919a_8200u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let changed = request.query(find("A", "sh")).execute().unwrap();
        assert_eq!(changed.rows().len(), 1);
        assert_eq!(changed.rows()[0].coordinate, coordinate);
        assert_eq!(changed.rows()[0].name, "shARED");
        assert!(
            matches!(initial.deliver_current(&request), Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause))) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded),
            "the original body-sensitive Source cannot take its current-output fast path"
        );
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let reused = settle(&world, &request, "A");
        assert_eq!(
            reused.settlement().root_settlement().posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(
            reused.settlement().root_producer_contacts_in_this_demand(),
            0
        );
        assert!(reused.settlement().root_receipt().is_none());
        let output = reused
            .read_current(&request, find("A", "produced:"))
            .unwrap();
        assert_eq!(output.rows(), &[original]);
        reused.deliver_current(&request).unwrap();
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
        // A fresh current accepted stable lineage continues its own authority.
        let again = settle(&world, &request, "A");
        assert_eq!(
            again.settlement().root_producer_contacts_in_this_demand(),
            0
        );
        assert_eq!(
            again
                .read_current(&request, find("A", "produced:"))
                .unwrap()
                .rows(),
            output.rows()
        );
        let changed_source = request.query(find("A", "sh")).execute().unwrap();
        let outcome = request
            .advanced()
            .mutate(super::super::super::super::mutation::Rename {
                directory: "A".into(),
                prefix: "sh".into(),
                item: "x".into(),
                replacement: "sham".into(),
            })
            .expect_result_set(changed_source.into_result_set_observation().unwrap())
            .idempotency(&0x919a_8201u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        assert!(
            matches!(again.deliver_current(&request), Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause))) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded),
            "conservative stable registration retains full Source verification after later edits"
        );
    });
}

#[test]
fn equal_scalar_input_cannot_reuse_a_replaced_full_generation_source() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
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
        let initial = settle(&world, &request, "A");
        let original = request.query(find("A", "sh")).execute().unwrap().rows()[0].clone();
        edit(
            &world,
            &request,
            Edit::RetireItem {
                directory: "A".into(),
                item: "x".into(),
            },
            0x919a_8210,
        );
        edit(
            &world,
            &request,
            Edit::Item {
                directory: "A".into(),
                identity: "x".into(),
                name: "shared".into(),
            },
            0x919a_8211,
        );
        let current = request.query(find("A", "sh")).execute().unwrap();
        assert_eq!(current.rows().len(), 1);
        assert_eq!(
            (&current.rows()[0].identity, &current.rows()[0].name),
            (&original.identity, &original.name)
        );
        assert_ne!(current.rows()[0].coordinate, original.coordinate);
        assert!(
            matches!(initial.deliver_current(&request), Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause))) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
        );
        initial_successor::refuse(&world, &request);
    });
}
