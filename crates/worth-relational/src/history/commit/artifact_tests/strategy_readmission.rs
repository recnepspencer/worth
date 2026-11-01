//! A real strategy-bearing artifact keeps each decoded shared leaf funded.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::history::data::PositionedCanonicalCommit;
use worth_execution::{
    CancellationSource, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
    SerialMemoryBudget, SerialRequest,
};

#[test]
fn strategy_input_clone_retains_only_its_original_decoded_bytes_grant() {
    let source = source_artifact(false);
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let (raw, actual) = decode(&source, &request);
    let original = actual
        .envelope()
        .strategy_artifacts
        .as_ref()
        .unwrap()
        .canonical_input();
    let retained = original.clone();
    assert!(std::ptr::eq(
        original.canonical_bytes(),
        retained.canonical_bytes()
    ));
    assert!(
        source.as_bytes().len() > retained.canonical_bytes().len() * 100,
        "the small input must not retain the huge independently owned index key"
    );
    let expected = arc_slice_bytes::<u8>(retained.canonical_bytes().len())
        + arc_slice_bytes::<ExecutionMemoryReservation>(1);
    assert!(expected > 0);
    drop((raw, actual));
    assert_eq!(budget.charged_memory_bytes(), expected);
    assert!(!retained.canonical_bytes().is_empty());
    drop(retained);
    assert_eq!(budget.charged_memory_bytes(), 0);
    refuse_zero_budget(&source);
}

#[test]
fn strategy_descriptor_clone_retains_only_its_original_decoded_locator_grant() {
    let source = source_artifact(true);
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let (raw, actual) = decode(&source, &request);
    let original = actual
        .envelope()
        .strategy_artifacts
        .as_ref()
        .unwrap()
        .merge_descriptor();
    let retained = original.clone();
    assert!(std::ptr::eq(
        original.intent_scope_targets(),
        retained.intent_scope_targets()
    ));
    let nested = retained
        .intent_scope_targets()
        .iter()
        .map(|locator| locator.owned_allocation_capacity_bytes() as u64)
        .sum::<u64>();
    assert!(
        nested > 0,
        "the real nested locator strings/path own storage"
    );
    let expected = arc_slice_bytes::<worth_foundational::facade::AspectFieldLocator>(
        retained.intent_scope_targets().len(),
    ) + arc_slice_bytes::<ExecutionMemoryReservation>(1)
        + nested;
    assert!(
        actual
            .envelope()
            .strategy_artifacts
            .as_ref()
            .unwrap()
            .canonical_input()
            .canonical_bytes()
            .len() as u64
            > expected * 100,
        "the small locator must not retain the huge input sibling"
    );
    assert!(expected > 0);
    drop((raw, actual));
    assert_eq!(budget.charged_memory_bytes(), expected);
    assert_eq!(retained.intent_scope_targets().len(), 1);
    drop(retained);
    assert_eq!(budget.charged_memory_bytes(), 0);
    refuse_zero_budget(&source);
}

fn decode(
    source: &Artifact,
    request: &ExecutionRequest<'_, '_>,
) -> (Artifact, PositionedCanonicalCommit) {
    let mut bytes =
        ExecutionRequestArrayBuilder::<u8>::allocate(*request, source.as_bytes().len()).unwrap();
    bytes.extend_from_slice(source.as_bytes()).unwrap();
    let raw = Artifact::from_owned_bytes(bytes.seal().unwrap().into_immutable_bytes()).unwrap();
    let positioned = raw.readmit_owned(request).unwrap().into_positioned();
    (raw, positioned)
}
fn refuse_zero_budget(source: &Artifact) {
    let budget = SerialMemoryBudget::new(Some(0));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    assert!(matches!(source.readmit_owned(&request), Err(crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(_))));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
fn arc_slice_bytes<T>(count: usize) -> u64 {
    // Independent concrete Arc layouts, not the production quote/control helper.
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::array::<T>(count).unwrap())
        .unwrap()
        .0
        .pad_to_align();
    u64::try_from(layout.size()).unwrap()
}
fn source_artifact(large_input: bool) -> Artifact {
    use crate::commit_strategies::data::{
        CommitStrategyId, StrategyCallerProvenance, StrategyRequestOrigin,
    };
    use crate::commit_strategies::strategies::{
        AspectFieldReconciliationInput, AspectFieldReconciliationStrategy,
    };
    use crate::tests::support::{aspect_field_locator, aspect_key, field_key, TestDurableMedium};
    let producer = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&producer);
    let descriptor = AspectFieldReconciliationStrategy::descriptor(CommitStrategyId(73));
    // Ordinary genuine source production is outside the bounded decode claim.
    let runtime = crate::runtime::builder::RelationalRuntimeBuilder::new()
        .schema_registry(crate::tests::support::test_schema_registry())
        .commit_strategy(
            AspectFieldReconciliationStrategy::registration(CommitStrategyId(73)).unwrap(),
        )
        .commit_strategy_executor(AspectFieldReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build_durable(TestDurableMedium::new().attachment())
        .unwrap();
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    runtime
        .index_authority()
        .register(crate::indexes::data::DerivedIndexDefinition {
            index_id: crate::indexes::data::DerivedIndexId(0),
            name: "original-leaf-size-adversary".into(),
            kind: crate::indexes::data::DerivedIndexKind::EntityField {
                field_locator: locator.clone(),
            },
            branch_scoped: false,
        })
        .unwrap();
    let large = "genuine-original-old-value".repeat(2048);
    let entity = crate::tests::support::create_entity(&request, &runtime, &large);
    let canonical = runtime
        .commit_strategies()
        .canonicalize_request(
            &AspectFieldReconciliationInput {
                entity_id: entity,
                field_locator: locator,
                desired_value: worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(if large_input {
                        large.clone() + "-changed"
                    } else {
                        "after".into()
                    }),
                ),
            }
            .into_native_canonical_request(StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            })
            .unwrap(),
        )
        .unwrap();
    let snapshot = runtime.visibility_authority().snapshot().unwrap();
    let execution = runtime
        .commit_strategies()
        .execute(&canonical, &snapshot, &request)
        .unwrap();
    let (input, mut authority) =
        crate::tests::support::test_owner_strategy_authority(&runtime, None);
    let lowered = authority
        .lower_execution_with_input(&request, &runtime, &canonical, &execution, input)
        .unwrap();
    let validated = authority
        .validate_lowered_plan(&request, &runtime, lowered)
        .unwrap();
    let committed = authority
        .execute_validated_commit(&request, &runtime, validated)
        .unwrap();
    let positioned = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .pop()
        .unwrap();
    assert_eq!(
        positioned.envelope().commit.commit_id,
        committed.commit.commit_id
    );
    assert!(
        positioned
            .envelope()
            .authored_index_writes
            .verified_entries()
            .unwrap()
            .iter()
            .any(|entry| entry
                .old_key
                .as_ref()
                .is_some_and(|key| key.owned_allocation_capacity_bytes() >= large.len() as u64)),
        "the other large leaf is an actual owner-issued old index key"
    );
    let artifact = Artifact::from_positioned(
        &positioned,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    crate::tests::support::release_test_commit_snapshot(&runtime, &committed);
    artifact
}
