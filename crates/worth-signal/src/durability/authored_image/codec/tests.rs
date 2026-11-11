use super::*;
use crate::data::aspect::{Aspect, AspectMask};
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::handle::NodeId;
use crate::data::node::*;
use crate::data::output::PartitionSubscription;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::reuse::{ArtifactSemanticBoundary, ReuseStrategy};
use crate::data::temporal::*;
use crate::logic::transaction::*;
use crate::schema::data::*;

#[test]
fn declaration_codec_preserves_nondefault_contracts_schema_policies_and_scopes() {
    let scope = PartitionSubscription::partition_and_detail("orders", "totals");
    let mut config = NodeEvaluationConfig {
        schema_binding: Some(SignalSchemaBinding::new(
            SignalSchemaId(17),
            SignalSchemaName::new("checkout"),
            SignalSchemaVersion::new(2, 3),
            "schema-meaning",
        )),
        merge_strategy_name: Some(MergeStrategyName::new("merge")),
        conflict_policy_name: Some(ConflictPolicyName::new("conflict")),
        identity_matcher_name: Some(IdentityMatcherName::new("identity")),
        source_only_policy_name: Some(SourceOnlyPolicyName::new("source")),
        deletion_policy_name: Some(DeletionPolicyName::new("deletion")),
        conflict_isolation_policy_name: Some(ConflictIsolationPolicyName::new("isolation")),
        aspect_merge_policy_bindings: vec![AspectMergePolicyBinding::new(
            Aspect::new(2),
            AspectMergePolicyName::new("aspect"),
        )],
        comparator: Some(VersionComparatorPolicy::Custom {
            key: "compare".into(),
        }),
        output_equivalence: OutputEquivalencePolicy::AspectVersionTolerance { epsilon: 33 },
        condition: EvaluationCondition::Temporal(TemporalCondition::interval(
            IntervalCondition::try_new(25)
                .unwrap()
                .with_anchor(IntervalAnchor::ExplicitTick(ClockTick::new(71)))
                .with_missed_tick_policy(MissedTickPolicy::CatchUpAll),
        )),
        partitioned_output: true,
        contract: NodeContract::reads(AspectMask::from_bits(3)),
    };
    config.contract.semantics.produces = AspectMask::from_bits(4);
    config.contract.semantics.partition_scope = Some(vec![scope.clone()]);
    config.contract.semantics.required_context = ContextRequirement::RelationalSnapshot;
    config.contract.projection.consumes = AspectMask::from_bits(5);
    config.contract.projection.consumes_partitions = Some(vec![scope.clone()]);
    config.contract.execution.bounded_inputs =
        Some(BoundedSignalInputs::new([DeclaredSignalInput::scoped(
            NodeId::new(29, 3),
            Aspect::new(1),
            scope,
        )]));
    config.contract.execution.max_checked_result_heap_bytes = Some(0);
    config.contract.execution.equivalence = EquivalenceContract {
        identity_basis: IdentityBasis::OutputIdentityAndContinuity,
        suppression_basis: SuppressionBasis::ContinuityToken,
        canonical_dependency_order: CanonicalDependencyOrder::SourceAspectScope,
        comparator_basis: ComparatorBasis::Tolerance,
    };
    config.contract.execution.path_class = PathClass::Rich;
    config.contract.execution.maintenance_mode = MaintenanceMode::IncrementalOnly;
    config.contract.execution.artifact_policy = ArtifactPolicyClass::ForensicReconstructable;
    config.contract.authority.policy = AuthorityPolicy::AuthoritativeOnly;
    config.contract.reuse.equivalence.required_boundaries =
        vec![ArtifactSemanticBoundary::CompositionRegionSet];
    config.contract.reuse.equivalence.supported_strategies =
        vec![ReuseStrategy::PartialArtifactSplicing];
    config
        .contract
        .reuse
        .equivalence
        .allows_snapshot_restore_reuse = true;
    config
        .contract
        .reuse
        .equivalence
        .allows_authority_reconciliation_reuse = true;
    config.contract.reuse.retain_certification = false;

    let mut bytes = Vec::new();
    configuration::write(
        &mut Write::new(branch(), &mut |part| {
            bytes.extend_from_slice(part);
            Ok(())
        }),
        &config,
    )
    .unwrap();
    let mut read = Read::new(&bytes, branch());
    read.materialize = true;
    assert_eq!(configuration::read(&mut read).unwrap(), config);
    read.finish().unwrap();
}

#[test]
fn authored_declarations_exclude_process_local_conditional_binding_counters() {
    fn bytes(value: &NodeDefinitionData) -> Vec<u8> {
        let mut result = Vec::new();
        write_definition(
            &mut Write::new(branch(), &mut |part| {
                result.extend_from_slice(part);
                Ok(())
            }),
            value,
        )
        .unwrap();
        result
    }
    let original = NodeDefinitionData::default();
    let mut rebound = original.clone();
    rebound.conditional_contract_generation = 91;
    rebound.conditional_contract_occurrence = 7_991;
    assert_eq!(bytes(&original), bytes(&rebound));
}

fn branch() -> worth_foundational::facade::DurableSignalBranchIdentity {
    use worth_foundational::facade::*;
    DurableSignalBranchIdentity::from_recorded(
        DurableSignalOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
            SignalOwnerSlotIdentity::from_recorded(1),
        ),
        0,
    )
}

#[test]
fn literal_threshold_payloads_round_trip_without_node_result_storage() {
    use worth_foundational::facade::*;
    let values = [
        AspectValue::Null,
        AspectValue::Bool(true),
        AspectValue::Int8(-3),
        AspectValue::Int16(-701),
        AspectValue::Int32(-91_001),
        AspectValue::Int64(i64::MIN),
        AspectValue::UInt8(255),
        AspectValue::UInt16(65_000),
        AspectValue::UInt32(4_000_000),
        AspectValue::UInt64(u64::MAX),
        AspectValue::Float32(CanonicalF32::from_f32(2.5)),
        AspectValue::Float64(CanonicalF64::from_f64(-3.75)),
        AspectValue::Decimal(CanonicalDecimal::new("12.50")),
        AspectValue::BigInt(CanonicalBigInt::new("12345678901234567890")),
        AspectValue::Rational(
            CanonicalRational::new(CanonicalBigInt::new("9"), CanonicalBigInt::new("17")).unwrap(),
        ),
        AspectValue::String(InternedString::Raw("authored literal".into())),
        AspectValue::String(InternedString::Stored(StoredValueText::new(
            "authored literal".into(),
        ))),
        AspectValue::String(InternedString::Symbol(Symbol(27))),
        AspectValue::Uuid([9; 16]),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: -73,
        }),
        AspectValue::Time(CanonicalTime::new(19).unwrap()),
        AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: -700,
        }),
        AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 900,
            offset_minutes: -120,
        }),
        AspectValue::EntityRef(EntityId::new(PartitionId(4), 700, 3)),
        AspectValue::ContentRef(ContentRefId([7; 32])),
        AspectValue::InlineBytes(InlineBytes::try_new(vec![0, 255, 17]).unwrap()),
    ];
    let mut literal_wire = None;
    for value in values {
        let mut bytes = Vec::new();
        threshold_value::write(
            &mut Write::new(branch(), &mut |part| {
                bytes.extend_from_slice(part);
                Ok(())
            }),
            &value,
        )
        .unwrap();
        if value.raw_text() == Some("authored literal") {
            if let Some(original) = &literal_wire {
                assert_eq!(
                    &bytes, original,
                    "stored text keeps Raw's existing wire grammar"
                );
            } else {
                literal_wire = Some(bytes.clone());
            }
        }
        let mut input = Read::new(&bytes, branch());
        input.materialize = true;
        assert_eq!(threshold_value::read(&mut input).unwrap(), value);
        input.finish().unwrap();
    }
}
#[test]
fn full_conditional_definition_round_trips_literal_parameters_and_provider_roles() {
    use crate::data::conditional_execution::*;
    use worth_foundational::facade::AspectValue;
    let definition = SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::DeltaThreshold(SignalDeltaThresholdContract::new(
            AspectValue::UInt64(35),
            "basis-points",
            SignalThresholdValueFamily::Integer,
            SignalThresholdComparisonDomain::RelativeRatio,
            SignalThresholdBoundary::Exclusive,
        )),
        dependency_aspects: AspectMask::from_bits(7),
        trigger_aspects: AspectMask::from_bits(3),
        dependency_comparator: SignalConditionalVersionComparator::Tolerance(8),
        output_comparator: SignalConditionalVersionComparator::RuntimeResolved,
        artifact_reuse: SignalConditionalArtifactReuse::RuntimeResolved,
    };
    let mut bytes = Vec::new();
    conditional_definition::write(
        &mut Write::new(branch(), &mut |part| {
            bytes.extend_from_slice(part);
            Ok(())
        }),
        &definition,
    )
    .unwrap();
    let mut input = Read::new(&bytes, branch());
    input.materialize = true;
    assert_eq!(
        conditional_definition::read(&mut input).unwrap(),
        definition
    );
    input.finish().unwrap();
}

#[test]
fn lowered_conditional_roles_cannot_replace_a_missing_definition() {
    use crate::data::comparator::{
        InstalledSignalComparatorIdentity, InstalledSignalComparatorRole,
    };
    let node = NodeId::new(0, 0);
    let comparator = InstalledSignalComparatorIdentity::new(
        17,
        node,
        InstalledSignalComparatorRole::OutputEquivalence,
    );
    let mut condition = NodeDefinitionData::default();
    condition.eval_config.condition = EvaluationCondition::Installed(
        InstalledSignalConditionIdentity::new(17, node, InstalledSignalConditionRole::Predicate),
    );
    let mut dependency = NodeDefinitionData::default();
    dependency.eval_config.comparator = Some(VersionComparatorPolicy::Installed {
        identity: InstalledSignalComparatorIdentity::new(
            17,
            node,
            InstalledSignalComparatorRole::DependencyVersion,
        ),
    });
    let mut output = NodeDefinitionData::default();
    output.eval_config.output_equivalence = OutputEquivalencePolicy::Installed {
        identity: crate::data::output_equivalence::InstalledOutputEquivalenceIdentity::from_installed_comparator(comparator).unwrap(),
    };
    for definition in [condition, dependency, output] {
        let mut bytes = Vec::new();
        assert!(matches!(
            write_definition(&mut Write::new(branch(), &mut |_| Ok(())), &definition),
            Err(Denial::ConditionalDefinitionMissing)
        ));
        let mut emit = |part: &[u8]| {
            bytes.extend_from_slice(part);
            Ok(())
        };
        let mut out = Write::new(branch(), &mut emit);
        out.boolean(false).unwrap();
        out.boolean(false).unwrap();
        configuration::write(&mut out, &definition.eval_config).unwrap();
        out.count(0).unwrap();
        let mut input = Read::new(&bytes, branch());
        input.materialize = true;
        assert!(matches!(
            graph_rows::read_definition(&mut input),
            Err(Denial::ConditionalDefinitionMissing)
        ));
    }
}
