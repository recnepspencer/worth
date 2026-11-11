//! Original Empty and canonical components remain source-usable through actual Home ACK.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::{BranchId, RelationalCommitReceipt},
    identity::{EntityId, KindId},
    runtime::{ProjectionAspectScope, RelationalRuntime},
    schema::SchemaAuthoritySnapshot,
};
use worth_runtime_world::facade::RuntimeWorldCancellationSource;
use worth_store_testkit_application_oracle::facade::world_home::WorldHomeFixture;

#[derive(Debug, PartialEq, Eq)]
struct OriginalComponent {
    schema: SchemaAuthoritySnapshot,
    schema_digest: [u8; 32],
    entities: Vec<EntityId>,
}

fn read_basis(
    native: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    execution: &ExecutionRequest<'_, '_>,
) -> OriginalComponent {
    let observation = basis.observation();
    let snapshot = native
        .snapshots()
        .snapshot_for_observation(&observation)
        .unwrap();
    let pin = native
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    let schema = pin.schema().authority_snapshot();
    let schema_digest = pin.schema().authority_digest_bytes();
    let view = native
        .read_truth()
        .project_observation(&observation)
        .unwrap();
    let entities = view
        .entity_records_with_projection_scope(
            execution,
            KindId(1),
            ProjectionAspectScope::whole_aspects([crate::batches::aspect_key("name")]),
            |record| Some(record.entity_id()),
        )
        .unwrap();
    drop(view);
    drop(pin);
    native.snapshots().release_snapshot(&snapshot).unwrap();
    OriginalComponent {
        schema,
        schema_digest,
        entities,
    }
}

fn read_original(
    fixture: &WorldHomeFixture,
    receipt: &RelationalCommitReceipt,
    execution: &ExecutionRequest<'_, '_>,
) -> OriginalComponent {
    let native = fixture.home.relational().runtime();
    let subjects = native
        .durability_authority()
        .discover_all_associated_effects(execution)
        .unwrap();
    let mut matching = subjects
        .iter()
        .filter(|subject| subject.receipt() == receipt);
    let subject = matching
        .next()
        .expect("actual original Native publication remains discoverable");
    assert!(matching.next().is_none());
    let effect = native
        .durability_authority()
        .readmit_associated_effect(execution, subject)
        .unwrap();
    let original = fixture
        .home
        .world()
        .observation_port()
        .readmitted_native_publication(execution, effect)
        .unwrap();
    read_basis(
        native,
        original.before().basis().relational_basis(),
        execution,
    )
}

fn retained_component(empty: bool) {
    use worth_store_application::ApplicationHomeCloseOutcome;
    let directory = tempfile::tempdir().unwrap();
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let fixture = super::coupled_home::workload(directory.path(), false, &execution);
    let expected_entities = if empty {
        Vec::new()
    } else {
        let seed = crate::workload::commit(
            fixture.home.relational().runtime(),
            &execution,
            &BranchId("main".into()),
            crate::batches::create(0),
        )
        .unwrap();
        let worth_relational::facade::transactions::RecordRef::Entity(entity) =
            seed.changed_records[0]
        else {
            panic!("canonical seed entity")
        };
        fixture
            .home
            .relational()
            .runtime()
            .snapshots()
            .release_snapshot(&seed.snapshot)
            .unwrap();
        vec![entity]
    };
    let bootstrap = fixture.bootstrap_root(&execution);
    let predecessor = bootstrap.product_branch().clone();
    let reference = read_basis(
        fixture.home.relational().runtime(),
        predecessor.basis().relational_basis(),
        &execution,
    );
    assert_eq!(reference.entities, expected_entities);
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        predecessor,
        crate::batches::create(1),
        &cancellation.token(),
    );
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    let receipt = performed
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt()
        .clone();
    drop(performed.consume());
    assert_eq!(read_original(&fixture, &receipt, &execution), reference);
    let selected = super::checkpoint::select_before_native_ack(&fixture.home, || {
        assert_eq!(read_original(&fixture, &receipt, &execution), reference);
    });
    let generation = selected.generation;
    drop(selected);
    assert_eq!(read_original(&fixture, &receipt, &execution), reference);
    let completed = fixture.home.recover_checkpoint(&execution).unwrap();
    assert_eq!(completed.generation(), generation);
    assert_eq!(read_original(&fixture, &receipt, &execution), reference);
    assert!(matches!(
        fixture
            .home
            .close(&execution)
            .unwrap_or_else(|refused| panic!("World close: {:?}", refused.reason())),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));

    let fixture = super::coupled_home::workload(directory.path(), false, &execution);
    assert_eq!(read_original(&fixture, &receipt, &execution), reference);
    let expected = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let prepared = fixture.prepare_publication(
        &execution,
        expected,
        crate::batches::create(2),
        &cancellation.token(),
    );
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    drop(performed.consume());
    assert_eq!(read_original(&fixture, &receipt, &execution), reference);
    assert!(matches!(
        fixture
            .home
            .close(&execution)
            .unwrap_or_else(|refused| panic!("World close: {:?}", refused.reason())),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));
}

#[test]
fn selected_empty_original_schema_remains_usable_after_observer_loss_reopen_and_advancement() {
    retained_component(true);
}
#[test]
fn selected_canonical_original_remains_usable_after_observer_loss_reopen_and_advancement() {
    retained_component(false);
}
