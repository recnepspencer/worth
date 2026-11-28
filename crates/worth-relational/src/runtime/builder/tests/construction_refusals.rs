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

#[test]
fn unsupported_declared_invariant_execution_point_refuses_runtime_construction() {
    use crate::validation::data::{
        InvariantCatalog, InvariantExecutionPoint, InvariantRegistration, InvariantRule,
    };
    let catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::snapshot_publication_blocking(
            InvariantRule::MaxMergedIntents(1),
        )],
    };
    let result = crate::facade::runtime::RelationalRuntimeApi::builder()
        .invariant_catalog(catalog)
        .build();
    assert!(matches!(
        result,
        Err(
            crate::runtime::RelationalRuntimeConstructionDenial::InvariantExecutionPoint {
                registration: 0,
                execution_point: InvariantExecutionPoint::SnapshotPublication
            }
        )
    ));
}
