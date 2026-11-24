//! Public runtime builder registration, canonicalization, and grouped setup overrides.
use worth_relational::facade::{
    commit_strategies::{
        CommitStrategyDescriptor, CommitStrategyFamilyName, CommitStrategyId,
        CommitStrategyRegistration, CommitStrategySemanticName, CommitStrategyVersion,
        NativeStrategyCommitRequest, PersistentArtifactName, StrategyCallerProvenance,
        StrategyInputSchemaName, StrategyInputSchemaVersion, StrategyIntentName,
        StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract,
        StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
        StrategyRequestOrigin, StrategyTraversalBasis,
    },
    config::{ConfigValueSource, DurableLogPolicy, DurableLogRetentionMode},
    diagnostics::RelationalDiagnosticsProfile,
    runtime::RelationalRuntimeBuilder,
    schema::RelationalSchemaRegistry,
};

#[test]
fn builder_attaches_frozen_commit_strategy_registry() {
    let registration = CommitStrategyRegistration::new(CommitStrategyDescriptor::new(
        CommitStrategyId(41),
        CommitStrategySemanticName::new("strategy.intent.reconcile"),
        CommitStrategyFamilyName::new("strategy.intent"),
        CommitStrategyVersion::new(1, 0),
        StrategyIntentName::new("reconcile.desired.state"),
        StrategyInputSchemaName::new("intent.reconcile.input.v1"),
        StrategyInputSchemaVersion(1),
        StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
        StrategyReadContract {
            scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
            locality_class: StrategyReadLocalityClass::SinglePartition,
            traversal_basis: StrategyTraversalBasis::NoTraversal,
            packet_contract: StrategyPacketContract::ProjectionOnly,
            cost_class: StrategyReadCostClass::ORequestedSurface,
        },
        PersistentArtifactName::new("strategy.intent.reconcile"),
    ))
    .expect("valid strategy registration");
    let runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration.clone())
        .build()
        .expect("valid native runtime configuration");

    assert_eq!(runtime.commit_strategy_registry().len(), 1);
    assert_eq!(
        runtime
            .commit_strategy_registry()
            .iter()
            .next()
            .expect("registered strategy")
            .descriptor()
            .semantic_name()
            .as_str(),
        "strategy.intent.reconcile"
    );
    assert!(!runtime
        .commit_strategy_registry()
        .registry_digest()
        .is_empty());
    assert_eq!(
        runtime
            .commit_strategy_registry()
            .iter()
            .next()
            .expect("registered strategy")
            .descriptor()
            .digest(),
        registration.descriptor().digest()
    );
    assert_eq!(
        runtime
            .config()
            .commit_strategies
            .registrations
            .first()
            .expect("config registration")
            .descriptor()
            .semantic_name()
            .as_str(),
        "strategy.intent.reconcile"
    );
    assert_eq!(
        runtime
            .config()
            .provenance
            .source_for("commit_strategies.registrations")
            .expect("strategy provenance")
            .source,
        ConfigValueSource::BuilderOverride
    );
    assert!(runtime
        .config()
        .provenance
        .source_for("commit_strategies.registrations")
        .expect("strategy provenance")
        .detail
        .contains("descriptor_set_digest="));
}

#[test]
fn runtime_commit_strategy_facade_canonicalizes_requests_against_frozen_registry() {
    let registration = CommitStrategyRegistration::new(CommitStrategyDescriptor::new(
        CommitStrategyId(41),
        CommitStrategySemanticName::new("strategy.intent.reconcile"),
        CommitStrategyFamilyName::new("strategy.intent"),
        CommitStrategyVersion::new(1, 0),
        StrategyIntentName::new("reconcile.desired.state"),
        StrategyInputSchemaName::new("intent.reconcile.input.v1"),
        StrategyInputSchemaVersion(1),
        StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
        StrategyReadContract {
            scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
            locality_class: StrategyReadLocalityClass::SinglePartition,
            traversal_basis: StrategyTraversalBasis::NoTraversal,
            packet_contract: StrategyPacketContract::ProjectionOnly,
            cost_class: StrategyReadCostClass::ORequestedSurface,
        },
        PersistentArtifactName::new("strategy.intent.reconcile"),
    ))
    .expect("valid strategy registration");
    let runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration)
        .build()
        .expect("valid native runtime configuration");

    let request = NativeStrategyCommitRequest::from_native_canonical_bytes(
        CommitStrategySemanticName::new("strategy.intent.reconcile"),
        b"native-request".to_vec(),
        StrategyCallerProvenance {
            request_origin: StrategyRequestOrigin::Api,
            actor_identity: Some("user-1".to_string()),
            correlation_id: Some("corr-1".to_string()),
        },
    );

    let canonical = runtime
        .commit_strategies()
        .canonicalize_request(&request)
        .expect("canonical request");

    assert_eq!(canonical.strategy_id(), CommitStrategyId(41));
    assert_eq!(
        canonical.canonical_input().canonical_bytes(),
        b"native-request"
    );
}

#[test]
fn builder_grouped_setup_sections_apply_overrides() {
    let runtime = RelationalRuntimeBuilder::new()
        .runtime_setup(|runtime| {
            runtime
                .runtime_name("grouped-runtime")
                .diagnostics(RelationalDiagnosticsProfile::geometry_operational_hot_path());
        })
        .schema_setup(|schema| {
            schema.schema_registry(RelationalSchemaRegistry::new());
        })
        .storage_setup(|storage| {
            storage.entity_capacity(42).relation_capacity(24);
        })
        .durability_setup(|durability| {
            durability.durable_log_policy(DurableLogPolicy {
                retention_mode: DurableLogRetentionMode::CompactAfterCheckpoint,
                max_in_memory_envelopes: 7,
                compact_after_checkpoint: true,
            });
        })
        .build()
        .expect("valid native runtime configuration");

    assert_eq!(runtime.config().execution.runtime_name, "grouped-runtime");
    assert_eq!(runtime.config().storage.initial_entity_capacity, 42);
    assert_eq!(runtime.config().storage.initial_relation_capacity, 24);
    assert_eq!(
        runtime
            .config()
            .durability
            .policy
            .log
            .max_in_memory_envelopes,
        7
    );
    assert_eq!(
        runtime.config().diagnostics.profile,
        RelationalDiagnosticsProfile::geometry_operational_hot_path()
    );
}
