use crate::facade;
use std::sync::OnceLock;
use worth_foundational::facade::{AspectKey, AspectValue};

fn public_api_projection_aspects() -> Vec<AspectKey> {
    static ASPECTS: OnceLock<Vec<AspectKey>> = OnceLock::new();
    ASPECTS
        .get_or_init(|| vec![AspectKey::new("name").unwrap()])
        .clone()
}

struct PublicApiProjection;

impl facade::runtime::EntityRecordProjection for PublicApiProjection {
    const KIND: facade::identity::KindId = facade::identity::KindId(1);

    fn projection_scope() -> facade::runtime::ProjectionAspectScope {
        facade::runtime::ProjectionAspectScope::whole_aspects(public_api_projection_aspects())
    }

    fn from_record(record: facade::runtime::EntityProjectionRecord<'_>) -> Option<Self> {
        let AspectValue::String(_) = record.aspect_value(&AspectKey::new("name").unwrap())? else {
            return None;
        };
        Some(Self)
    }
}

#[test]
fn facade_namespaces_expose_domain_groupings() {
    let _branch: facade::history::BranchId = facade::history::BranchId("main".to_string());
    let _entity: facade::identity::EntityId =
        facade::identity::EntityId::new(facade::identity::PartitionId::main(), 1, 0);
    let _config = facade::config::RelationalRuntimeProfile::CertificationCore;
    let runtime = facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(facade::schema::RelationalSchemaRegistry::new())
        .build();
    let _txn_options =
        crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime);
    let _durability_mode = facade::durability::DurabilityMode::InMemoryCanonical;
    let _diagnostics_scope = facade::diagnostics::DiagnosticsScope::Transaction;
    let _patch_mode = facade::publication::PatchPublicationMode::CommitNative;
    let _snapshot_policy = facade::snapshots::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation;
    let _projection_kind = <PublicApiProjection as facade::runtime::EntityRecordProjection>::KIND;
    let _projection_scope =
        <PublicApiProjection as facade::runtime::EntityRecordProjection>::projection_scope();
}

#[test]
fn admitted_observation_opens_its_selected_root_after_branch_movement() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = public_api_runtime();
    let first = crate::tests::support::create_entity_outcome(execution, &runtime, "observed");
    let first_version = first.version_id;
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    assert!(runtime
        .snapshots()
        .release_snapshot(&first.snapshot)
        .is_ok());

    let second = crate::tests::support::create_entity_outcome(execution, &runtime, "later");
    let observed = runtime
        .snapshots()
        .snapshot_for_observation(&observation)
        .expect("admitted observation keeps its exact immutable root");

    assert_eq!(observed.version_id, first_version);
    assert_eq!(observed.branch_id, *identity.branch_id());
    assert!(runtime
        .read_truth()
        .read_snapshot(&observed, execution)
        .expect("read request remains live")
        .is_some());
    assert!(runtime.snapshots().release_snapshot(&observed).is_ok());
    assert!(runtime
        .snapshots()
        .release_snapshot(&second.snapshot)
        .is_ok());
}

#[test]
fn foreign_runtime_identity_cannot_admit_a_branch_observation() {
    let runtime = public_api_runtime();
    let foreign = public_api_runtime();
    let foreign_identity = foreign.main_branch_identity();

    assert!(matches!(
        runtime.observe_branch(&foreign_identity),
        Err(facade::branch::RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
}

#[test]
fn admitted_basis_clones_share_one_descriptor_and_observation_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = public_api_runtime();
    let committed =
        crate::tests::support::create_entity_outcome(execution, &runtime, "shared-basis");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let cloned = basis.clone();

    assert_eq!(basis.descriptor(), cloned.descriptor());
    assert_eq!(descriptor, *basis.descriptor());
    assert_eq!(basis.observation().version_id(), committed.version_id);
    assert_eq!(
        basis.observation().selected_root_identity(),
        cloned.observation().selected_root_identity()
    );
}

#[test]
fn main_movement_does_not_change_an_inherited_child_observation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = public_api_runtime();
    let baseline =
        crate::tests::support::create_entity_outcome(execution, &runtime, "child-baseline");
    let baseline_version = baseline.snapshot.version_id;
    let main = runtime.main_branch_identity();
    let (_, source_basis) = runtime
        .observe_fork_source(main.branch_id())
        .expect("main exposes an owner-issued fork basis");
    let child_id = facade::history::BranchId("observation-child".to_owned());
    runtime
        .fork_branch(child_id.clone(), source_basis)
        .expect("child retains the inherited immutable root");
    crate::tests::support::create_entity_outcome(execution, &runtime, "main-advances");
    let child = runtime.branch_identity(&child_id).unwrap();
    let (_, child_basis) = runtime.observe_branch(&child).unwrap();

    assert_eq!(child_basis.observation().version_id(), baseline_version);
    assert_eq!(child_basis.identity().branch_id(), &child_id);
}

fn public_api_runtime() -> facade::runtime::RelationalRuntime {
    crate::tests::support::runtime_with_test_schema()
}
