use super::*;
use crate::data::aspect::{Aspect, AspectMask};
use crate::data::comparator::VersionComparatorPolicy;
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
        &mut Write::new(&mut |part| {
            bytes.extend_from_slice(part);
            Ok(())
        }),
        &config,
    )
    .unwrap();
    let mut read = Read::new(&bytes);
    read.materialize = true;
    assert_eq!(configuration::read(&mut read).unwrap(), config);
    read.finish().unwrap();
}
