use std::sync::Arc;

use super::RelationalRuntimeBuilder;
use crate::commit_strategies::data::{
    CommitStrategyDescriptor, CommitStrategyFamilyName, CommitStrategyId,
    CommitStrategyRegistration, CommitStrategySemanticName, CommitStrategyVersion,
    PersistentArtifactName, StrategyInputSchemaName, StrategyInputSchemaVersion,
    StrategyIntentName, StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract,
    StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
    StrategyTraversalBasis,
};
use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::durability::data::DurabilityMode;
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

struct BuilderTestRule;

impl CustomInvariantRule for BuilderTestRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("builder.test.rule"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Builder Test Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(1).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract::default(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn builder_attaches_custom_invariants_without_polluting_config() {
    let registration = CustomInvariantRegistration::new(BuilderTestRule).unwrap();
    let runtime = RelationalRuntimeBuilder::new()
        .custom_invariant(registration)
        .build();

    assert_eq!(
        runtime.config.schema.invariant_catalog.registrations.len(),
        2
    );
    assert_eq!(
        runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .len(),
        1
    );
    assert_eq!(
        runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .next()
            .unwrap()
            .rule_id()
            .as_str(),
        "builder.test.rule"
    );
}

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
        .build();

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
        crate::config::data::ConfigValueSource::BuilderOverride
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
        .build();

    let request =
        crate::facade::commit_strategies::NativeStrategyCommitRequest::from_native_canonical_bytes(
            CommitStrategySemanticName::new("strategy.intent.reconcile"),
            b"native-request".to_vec(),
            crate::facade::commit_strategies::StrategyCallerProvenance {
                request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Api,
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
                .execution_model(crate::config::data::RelationalExecutionModel::SingleLaneExecution)
                .diagnostics(RelationalDiagnosticsProfile::geometry_operational_hot_path());
        })
        .schema_setup(|schema| {
            schema.schema_registry(RelationalSchemaRegistry::new());
        })
        .storage_setup(|storage| {
            storage.entity_capacity(42).relation_capacity(24);
        })
        .durability_setup(|durability| {
            durability.durability_mode(DurabilityMode::InMemoryCanonical);
        })
        .build();

    assert_eq!(runtime.config().execution.runtime_name, "grouped-runtime");
    assert_eq!(runtime.config().storage.initial_entity_capacity, 42);
    assert_eq!(runtime.config().storage.initial_relation_capacity, 24);
    assert_eq!(
        runtime.config().durability.policy.mode,
        DurabilityMode::InMemoryCanonical
    );
    assert_eq!(
        runtime.config().diagnostics.profile,
        RelationalDiagnosticsProfile::geometry_operational_hot_path()
    );
}
