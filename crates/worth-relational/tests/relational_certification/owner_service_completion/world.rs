use worth_relational::facade::branch::{
    RelationalBranchIdentity, RelationalForkOutcome, RelationalOwnerServicePorts,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::transactions::{CommitResult, WorkerIntentBatch};

pub(super) fn empty_runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .build()
}

pub(super) fn commit_through_services(
    runtime: &RelationalRuntime,
    services: &RelationalOwnerServicePorts,
    label: &str,
) -> CommitResult {
    let identity = runtime.main_branch_identity();
    let basis = services
        .basis_port()
        .admit_branch_basis(&identity)
        .expect("owner service admits the exact owner-issued basis");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("the admitted basis opens the canonical transaction path");
    transaction
        .push_batch(WorkerIntentBatch::new(label))
        .expect("the empty batch remains within its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(transaction)
        .expect("preparation service produces the canonical candidate");
    let performed = match services.publication_port().compare_and_publish(candidate) {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("publication service did not perform: {outcome:?}"),
    };
    services
        .settlement_port()
        .settle_performed_publication(performed)
        .expect("settlement service completes the performed publication")
}

pub(super) fn commit_direct(runtime: &RelationalRuntime, label: &str) -> CommitResult {
    let identity = runtime.main_branch_identity();
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("direct owner path admits its exact basis");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("direct admitted basis opens a transaction");
    transaction
        .push_batch(WorkerIntentBatch::new(label))
        .expect("the direct empty batch remains within budget");
    transaction
        .commit(runtime)
        .expect("the compatibility path commits canonically")
}

pub(super) fn fork_through_services(
    services: &RelationalOwnerServicePorts,
    target: &str,
) -> RelationalForkOutcome {
    let fork = services.fork_port();
    let (_, source) = fork
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("committed main supplies an exact fork basis");
    fork.fork_branch(BranchId(target.to_owned()), source)
        .expect("fork service installs the exact target")
}

pub(super) fn fork_direct(runtime: &RelationalRuntime, target: &str) -> RelationalForkOutcome {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("committed main supplies a direct fork basis");
    runtime
        .fork_branch(BranchId(target.to_owned()), source)
        .expect("direct compatibility path installs the exact target")
}

pub(super) fn committed_fork(
    target: &str,
) -> (
    RelationalRuntime,
    RelationalOwnerServicePorts,
    RelationalBranchIdentity,
) {
    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    commit_through_services(&runtime, &services, "owner-service-world");
    let identity = fork_through_services(&services, target)
        .target_identity()
        .clone();
    (runtime, services, identity)
}
