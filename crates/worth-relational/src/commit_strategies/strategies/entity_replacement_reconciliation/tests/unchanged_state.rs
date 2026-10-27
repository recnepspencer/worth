use super::*;

#[test]
fn entity_replacement_reconciliation_strategy_noops_when_authoritative_fields_match() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(78),
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_registry())
        .commit_strategy(
            EntityReplacementReconciliationStrategy::registration(descriptor.id())
                .expect("registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "service");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &EntityReplacementReconciliationInput {
                entity_id: entity,
                replacement_client_key: String::new(),
                desired_aspect_fields: AspectFieldPatch::from_locator(
                    name_patch_target(),
                    AspectValue::String(InternedString::Raw("service".to_string())),
                ),
            }
            .into_native_canonical_request(StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            })
            .expect("native canonical strategy request"),
        )
        .expect("canonical request");
    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("strategy execution");

    let output =
        EntityReplacementReconciliationOutput::decode(execution.output().canonical_bytes())
            .expect("decode output");
    assert_eq!(
        output.action,
        EntityReplacementReconciliationAction::NoChange
    );
    assert_eq!(execution.mutation_program().total_intent_count(), 0);
}
