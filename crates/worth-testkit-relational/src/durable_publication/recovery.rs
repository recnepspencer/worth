use super::*;
use crate::aspect_storage::publications::release_test_commit_snapshot;
pub fn source_journey(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    Arc<TestDurableMedium>,
    Recorded,
    Recorded,
    Association,
    Arc<worth_relational::facade::transactions::CommitResult>,
) {
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let predecessor = current(execution, &runtime);
    let association = Association::from_bytes(b"original-release-subject\0\xff").unwrap();
    let performed = perform(
        execution,
        &runtime,
        &association,
        "original-associated-effect",
    );
    let original_basis = performed.next_basis().clone();
    let result = runtime.settle_performed_publication(performed).unwrap();
    let semantic = Arc::new(result);
    release_test_commit_snapshot(&runtime, &semantic);
    create_entity(execution, &runtime, "later-native-head");
    let successor = runtime
        .owner_component_services()
        .basis_port()
        .export_recorded_retained_component(execution, &original_basis)
        .unwrap();
    (
        runtime,
        medium,
        predecessor,
        successor,
        association,
        semantic,
    )
}
