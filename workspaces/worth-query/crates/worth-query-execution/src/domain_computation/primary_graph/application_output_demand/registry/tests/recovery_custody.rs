use super::super::{ProgramRecoveryCustody, WorthQueryOutputDemandRegistry};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

fn occurrence_and_commit() -> (
    worth_runtime_world::facade::ProductBranchIncarnation,
    worth_runtime_world::facade::CompositeCommitIdentity,
) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture's default product occurrence is live");
    (
        product.observation().lifecycle_incarnation(),
        product.observation().selected_commit().clone(),
    )
}

fn recovery_custody(
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
) -> ProgramRecoveryCustody {
    ProgramRecoveryCustody {
        provider_runtime_instance_id: 1,
        product_occurrence: occurrence,
        source_commit,
        inventory: std::any::TypeId::of::<()>(),
        root: std::any::TypeId::of::<()>(),
        demand: Box::new(()),
    }
}

#[test]
fn recovery_custody_cannot_cross_occurrence_retirement_in_either_order() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (occurrence, commit) = occurrence_and_commit();
    let preparation = registry.begin_source_preparation(occurrence);
    registry.release_product_occurrence(occurrence);
    let denial = WorthQueryOutputDemandRegistry::retain_program_recovery_custody(
        &mut registry.state.lock().unwrap(),
        &preparation,
        recovery_custody(occurrence, commit),
    )
    .expect_err("retirement must close a pending custody transfer");
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Closed);
    assert!(registry.state.lock().unwrap().program_recovery.is_empty());

    let registry = WorthQueryOutputDemandRegistry::default();
    let (occurrence, commit) = occurrence_and_commit();
    let preparation = registry.begin_source_preparation(occurrence);
    WorthQueryOutputDemandRegistry::retain_program_recovery_custody(
        &mut registry.state.lock().unwrap(),
        &preparation,
        recovery_custody(occurrence, commit),
    )
    .expect("live preparation admits custody");
    registry.release_product_occurrence(occurrence);
    assert!(registry.state.lock().unwrap().program_recovery.is_empty());
}

#[test]
fn recovery_preparation_cannot_authorize_a_foreign_occurrence() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (authorized, _) = occurrence_and_commit();
    let (foreign, foreign_commit) = occurrence_and_commit();
    let preparation = registry.begin_source_preparation(authorized);
    let denial = WorthQueryOutputDemandRegistry::retain_program_recovery_custody(
        &mut registry.state.lock().unwrap(),
        &preparation,
        recovery_custody(foreign, foreign_commit),
    )
    .expect_err("preparation cannot transfer custody for another occurrence");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ForeignSource
    );
    assert_eq!(
        denial.subject(),
        "program recovery preparation belongs to another occurrence"
    );
    assert!(registry.state.lock().unwrap().program_recovery.is_empty());
}
