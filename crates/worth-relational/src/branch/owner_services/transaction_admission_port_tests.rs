use crate::mvcc::{RelationalBranchTransactionAdmissionDenial, RelationalTransactionIntent};
use crate::tests::support::{create_entity, runtime_with_test_schema};

#[test]
fn owner_port_routes_transaction_admission_and_releases_retention_on_drop() {
    let runtime = runtime_with_test_schema();
    let services = runtime.owner_component_services();
    let identity = runtime.main_branch_identity();
    let (_, basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the owner basis port issues the exact branch basis");
    let before = runtime.retention_cost_counters();

    let transaction = services
        .transaction_admission_port()
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("the owner transaction port admits the exact basis");
    let admitted = runtime.retention_cost_counters();
    assert_eq!(
        admitted.transaction_acquires,
        before.transaction_acquires + 1
    );
    drop(transaction);
    let released = runtime.retention_cost_counters();
    assert_eq!(
        released.transaction_releases,
        admitted.transaction_releases + 1
    );
}

#[test]
fn owner_port_rejects_a_basis_from_a_foreign_runtime() {
    let runtime = runtime_with_test_schema();
    let foreign_runtime = runtime_with_test_schema();
    let foreign_basis = foreign_runtime
        .owner_component_services()
        .basis_port()
        .observe_branch(&foreign_runtime.main_branch_identity())
        .expect("the foreign runtime issues its own exact basis")
        .1;

    assert!(matches!(
        runtime
            .owner_component_services()
            .transaction_admission_port()
            .begin_branch_transaction(&foreign_basis, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::ForeignRuntime { .. })
    ));
}

#[test]
fn owner_port_rejects_an_exact_basis_after_the_branch_moves() {
    let runtime = runtime_with_test_schema();
    let services = runtime.owner_component_services();
    let identity = runtime.main_branch_identity();
    let (_, basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the owner basis port issues the pre-movement basis");
    create_entity(&runtime, "transaction-admission-stale-basis");

    assert!(matches!(
        services
            .transaction_admission_port()
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::StaleBasis)
    ));
}

#[test]
fn owner_port_denies_after_the_runtime_owner_closes() {
    let (services, basis) = {
        let runtime = runtime_with_test_schema();
        let services = runtime.owner_component_services();
        let basis = services
            .basis_port()
            .observe_branch(&runtime.main_branch_identity())
            .expect("the owner issues a basis before close")
            .1;
        (services, basis)
    };

    assert!(matches!(
        services
            .transaction_admission_port()
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::OwnerUnavailable)
    ));
}
