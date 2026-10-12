use super::*;

#[test]
fn ordinary_continuity_reuses_registry_allocation_at_small_and_large_schema_scale() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for additional_kind_count in [0_u32, 256] {
        let runtime = runtime_with_registry_width(additional_kind_count);
        create_entity_outcome(execution, &runtime, "schema-allocation-basis");
        let prior_root = runtime
            .history
            .branch_cell(&BranchId("main".to_owned()))
            .and_then(|cell| cell.root())
            .expect("basis commit installs an exact branch root");
        let prior_registry = prior_root.schema_authority().registry_arc();
        let prior_authority_id = prior_root.schema_authority().allocation_id();
        let options = test_owner_transaction_validation_input_for_main(&runtime);

        let preparation = runtime.preparation_runtime_snapshot();
        let plan = crate::authority::commit::phases::schema_continuity::resolve_schema_continuity(
            &preparation,
            &BranchId("main".to_owned()),
            &options,
        )
        .expect("ordinary continuity resolves from the admitted branch root");
        let carried_registry = plan
            .target_schema_registry
            .as_ref()
            .expect("ordinary continuity carries its exact registry");
        assert!(
            Arc::ptr_eq(&prior_registry, carried_registry),
            "ordinary continuity must share the root registry allocation at width {additional_kind_count}"
        );

        create_entity_outcome(execution, &runtime, "schema-allocation-next");
        let next_root = runtime
            .history
            .branch_cell(&BranchId("main".to_owned()))
            .and_then(|cell| cell.root())
            .expect("ordinary publication installs its complete root");
        assert_eq!(
            next_root.schema_authority().allocation_id(),
            prior_authority_id
        );
        assert_eq!(next_root.publication_cost().new_schema_authorities, 0);
        assert_eq!(next_root.publication_cost().reused_schema_authorities, 1);
    }
}

fn runtime_with_registry_width(additional_kind_count: u32) -> RelationalRuntime {
    let mut registry = test_schema_registry();
    for offset in 0..additional_kind_count {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(10_000 + offset),
                kind_name: format!("allocation.probe.{offset}"),
                schema_id: SchemaId("test".to_owned()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .expect("allocation-scale schema kinds are unique and valid");
    }
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
}
