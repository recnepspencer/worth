//! Invalid caller configuration must be refused before service or dispatch.
use super::*;
use crate::runtime::{RelationalRuntime, RelationalRuntimeConstructionDenial as Denial};

#[test]
fn duplicate_custom_invariants_preserve_the_registration_cause() {
    let registration = CustomInvariantRegistration::new(BuilderTestRule).unwrap();
    let result = RelationalRuntimeBuilder::new()
        .custom_invariant(registration.clone())
        .custom_invariant(registration)
        .build();
    let Err(Denial::CustomInvariant(cause)) = result else {
        panic!("expected registration refusal")
    };
    assert_eq!(
        cause.identity.rule_id,
        CustomInvariantRuleId::new("builder.test.rule")
    );
    assert_eq!(
        cause.execution_point,
        InvariantExecutionPoint::CommitBoundary
    );
}

#[test]
fn invalid_main_branch_and_symbol_configuration_refuse_native_construction() {
    let runtime = RelationalRuntimeBuilder::new().build().unwrap();
    let mut config = runtime.config();
    config.history.main_branch = crate::history::data::BranchId(" ".into());
    assert!(matches!(
        RelationalRuntime::new(config),
        Err(Denial::MainBranch(
            crate::branch::RelationalBranchObservationConstructionDenial::EmptyBranchName
        ))
    ));
    let mut config = runtime.config();
    config.identity.symbol_table.entries = vec![
        (crate::symbols::data::Symbol(1), "first".into()),
        (crate::symbols::data::Symbol(1), "second".into()),
    ];
    assert!(matches!(
        RelationalRuntime::new(config),
        Err(Denial::SymbolTable(_))
    ));
}

fn strategy() -> CommitStrategyRegistration {
    CommitStrategyRegistration::new(CommitStrategyDescriptor::new(
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
    .expect("valid strategy registration")
}

#[test]
fn duplicate_strategy_descriptor_is_refused() {
    let registration = strategy();
    assert!(matches!(
        RelationalRuntimeBuilder::new()
            .commit_strategy(registration.clone())
            .commit_strategy(registration)
            .build(),
        Err(Denial::CommitStrategy(
            crate::commit_strategies::DuplicateCommitStrategyRegistration::Id {
                id: CommitStrategyId(41),
            }
        ))
    ));
}

#[test]
fn executor_registration_refuses_before_dispatch() {
    use crate::commit_strategies::strategies::IntentReconciliationStrategy;
    use crate::commit_strategies::CommitStrategyExecutionRegistryError as Cause;
    let strategy = strategy();
    let binding = IntentReconciliationStrategy::execution_registration(strategy.descriptor());
    assert!(matches!(
        RelationalRuntimeBuilder::new()
            .commit_strategy_executor(binding.clone())
            .build(),
        Err(Denial::CommitStrategyExecutor(
            Cause::MissingDescriptorRegistration { .. }
        ))
    ));
    assert!(matches!(
        RelationalRuntimeBuilder::new()
            .commit_strategy(strategy)
            .commit_strategy_executor(binding.clone())
            .commit_strategy_executor(binding)
            .build(),
        Err(Denial::CommitStrategyExecutor(
            Cause::DuplicateBinding { .. }
        ))
    ));
}
