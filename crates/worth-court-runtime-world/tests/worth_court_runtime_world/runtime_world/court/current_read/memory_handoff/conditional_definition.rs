use super::*;
use worth_runtime_bridge::facade::*;

#[test]
fn conditional_bridge_definition_follows_the_same_one_use_cutover_and_preserves_caller_error() {
    let fixture = crate::owned_async::ordinary_context::fixture::AsyncSourceCourt::new();
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    // Use the existing Native fixture and its real Bridge source. This World
    // owns a separate sealed conditional graph and its publication capability.
    let bridge = RuntimeBridge::builder()
        .with_relational_source(fixture.source.clone())
        .with_signal_sink(crate::owned_async::ordinary_context::fixture::NoDelivery)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("cutover-definition"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("cutover-record"),
                worth_foundational::facade::AspectKey::new("value").unwrap(),
                worth_foundational::facade::FieldKey::new("value").unwrap(),
            ),
            SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("value").unwrap(),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("cutover-definition"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .unwrap();
    let mut builder = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap();
    let lowering = builder.install_owned_conditional(definition(1)).unwrap();
    let mut bridge = builder.seal().unwrap();
    let world = RuntimeWorldOwner::builder()
        .with_relational_services(fixture.relational.owner_component_services())
        .with_signal_services(bridge.signal_owner_services())
        .with_signal_definition_publication(
            bridge
                .take_runtime_world_signal_definition_publication()
                .unwrap(),
        )
        .with_bridge_correspondence(bridge.runtime_world_correspondence_port())
        .with_budgets(RuntimeWorldBudgets::without_policy_limits())
        .with_clock(RuntimeWorldClock::from_source(
            crate::owned_async::ordinary_context::fixture::Clock,
        ))
        .with_execution_policy(match fixture.world.execution_placement() {
            RuntimeWorldExecutionPlacement::Serial(policy)
            | RuntimeWorldExecutionPlacement::Leased { policy, .. } => policy,
        })
        .build()
        .unwrap();
    let RuntimeWorldBootstrapOutcome::Performed(root) = world
        .lifecycle_port()
        .bootstrap_root(RuntimeWorldBootstrapIntent::new(
            ProductBranchCreationIntent::named("conditional-cutover").unwrap(),
            fixture.empty.basis().relational_basis().clone(),
            bridge.admitted_signal_basis().clone(),
            bridge.admitted_runtime_world_correspondence_basis().clone(),
        ))
        .unwrap()
    else {
        panic!("real sealed conditional World bootstrap")
    };
    let head = root.product_branch().clone();
    // Establish the committed source required by current-read publication.
    let token = RuntimeWorldCancellationSource::new().token();
    let prepared = world
        .publication_port()
        .prepare_without_signal(
            head.clone(),
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(fixture.prepare(&execution, &head)),
            &token,
            None,
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(warmup) = world
        .publication_port()
        .execute_without_signal(execution, prepared, &token)
    else {
        panic!("committed Native source")
    };
    drop(warmup.consume());
    let head = world
        .observation_port()
        .observe_product_branch(head.branch_identity())
        .unwrap();
    let predecessor = bridge
        .admit_conditional_signal_basis(&lowering, head.basis().signal_basis())
        .unwrap();
    let prepared_bridge = bridge
        .prepare_owned_conditional_definition_successor(&predecessor, definition(2))
        .unwrap();
    let prepared = world
        .publication_port()
        .prepare_with_signal(
            head.clone(),
            intent(
                CompositePublicationIntent::with_signal(Some(
                    RelationalTransactionIntent::ordinary(),
                ))
                .with_prepared_relational_candidate(fixture.prepare(&execution, &head)),
                &head,
            ),
            &token,
            None,
        )
        .unwrap();
    let (caller, outcome) = world
        .publication_port()
        .publish_bridge_conditional_definition_with_memory_cutover(
            execution,
            prepared,
            prepared_bridge,
            &bridge,
            &token,
            |cutover| {
                assert_eq!(cutover.current_read(), &head);
                assert_eq!(cutover.expected_target(), &head);
                cutover.perform();
                Err::<(), _>("caller status")
            },
        );
    assert_eq!(caller, Some(Err("caller status")));
    let RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
        publication,
        lowering: successor,
    } = outcome
    else {
        panic!("actual conditional publication cannot be erased by caller error")
    };
    let selected = world
        .observation_port()
        .observe_product_branch(head.branch_identity())
        .unwrap();
    assert_ne!(
        selected.basis().relational_basis(),
        head.basis().relational_basis()
    );
    assert_ne!(
        selected.basis().signal_basis().admission_identity(),
        head.basis().signal_basis().admission_identity()
    );
    assert_eq!(
        successor.signal_definition_generation(),
        lowering.signal_definition_generation() + 1
    );
    bridge
        .admit_conditional_signal_basis(&successor, selected.basis().signal_basis())
        .unwrap();
    assert!(bridge
        .admit_conditional_signal_basis(&lowering, selected.basis().signal_basis())
        .is_err());
    drop(publication.consume());
}

fn definition(version: u64) -> BridgeOwnedConditionalInstallationRequest {
    BridgeOwnedConditionalInstallationRequest {
        contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
            identity: std::sync::Arc::from("cutover-definition"),
            dependency_count: 0,
            condition_dependency_ordinals: Vec::new(),
            condition: BridgeConditionalCondition::Always,
            dependency_comparator: worth_signal::facade::SignalConditionalVersionComparator::Exact,
            output_comparator: worth_signal::facade::SignalConditionalVersionComparator::Exact,
            artifact_reuse: worth_signal::facade::SignalConditionalArtifactReuse::NotReusable,
        }),
        location: BridgeConditionalLocation::operation("cutover-definition"),
        dependencies: Vec::new(),
        providers: BridgeConditionalProviderSet::new().compute(Compute(version)),
    }
}

struct Compute(u64);
impl BridgeConditionalProviderSemantics for Compute {
    type SemanticContract = u64;
    fn semantic_contract(&self) -> u64 {
        self.0
    }
    fn retained_heap_bytes(
        &self,
        _: &u64,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::none())
    }
}
impl BridgeConditionalComputeProvider for Compute {
    fn compute(
        &self,
        _: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                self.0,
            )]),
        ))
    }
}
