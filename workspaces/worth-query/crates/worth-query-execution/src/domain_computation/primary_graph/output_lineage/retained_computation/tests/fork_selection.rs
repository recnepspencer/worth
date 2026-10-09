//! Exact fork horizons select one prior, and local absence ends inheritance.
use super::*;
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationSource, SignalBranchCreationPlan,
};

#[test]
fn nested_forks_select_the_captured_generation_and_stop_at_local_absence() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let make = |source: &crate::basis::WorthQueryProductBranchLease, name| {
        let intent = ProductBranchCreationIntent::from_source(
            name,
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ReuseExact,
            ),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = world
            .application
            .product_runtime()
            .create_product_branch(
                source,
                None,
                intent,
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("fork performs");
        };
        child
    };
    let child = make(&product, "computation-child");
    let admitted_child = world
        .application
        .product_runtime()
        .lease_from_observation(child.clone())
        .unwrap();
    let nested = make(&admitted_child, "computation-nested");
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: TypeId::of::<()>(),
    };
    let parent = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let partition = [0x71; 32];
    let restored =
        |lineage: &mut WorthQueryApplicationOutputLineage,
         observation: &worth_runtime_world::facade::ProductBranchObservation| {
            lineage.record_restoration(source.output_binding, source.runtime_authority, source.schema.clone(), source.scope, observation,
            Arc::new(super::super::super::WorthQueryApplicationOutputCorrespondence::default()),
            super::super::super::RecordedSourceIdentity::Checkpoint(crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x72; 32])),
            partition, None, [0x73; 32], super::super::super::ComputationSourceEvidence::for_test(false).retain_facts(Arc::from([])), None, None);
        };
    restored(&mut lineage, product.observation());
    let captured =
        Arc::clone(&lineage.by_source[&source][&parent.occurrence][&parent.generation][0]);
    captured.get().unwrap().mutable.lock().unwrap().computation = lineage.retain_computation(
        sealed_run_for_lineage_test(),
        None,
        None,
        lineage.prepay_computation_fork_scan_for_test(),
    );
    let before_forks = lineage.prepay_computation_fork_scan_for_test();
    lineage.register_fork(product.observation(), &child);
    lineage.register_fork(admitted_child.observation(), &nested);
    let select =
        |lineage: &WorthQueryApplicationOutputLineage,
         observation: &worth_runtime_world::facade::ProductBranchObservation| {
            lineage
                .computation_cell_in_ancestry(
                    &source,
                    ProductCoordinate {
                        occurrence: observation.lifecycle_incarnation(),
                        generation: observation.reference_generation().get(),
                    },
                    partition,
                    &mut InvalidationEditAdmission::new(
                        worth_relational::facade::mvcc::CompanionPreflightBudget {
                            maximum_work_visits: 4096,
                            maximum_preparation_bytes: 4096,
                        },
                    ),
                )
                .unwrap()
                .unwrap()
        };
    assert!(Arc::ptr_eq(&select(&lineage, &child), &captured));
    assert!(Arc::ptr_eq(&select(&lineage, &nested), &captured));
    assert!(lineage.fork_pins_computation(captured.get().unwrap()));
    let bytes = {
        let row = captured.get().unwrap().mutable.lock().unwrap();
        let RecordedComputation::Retained(state) = &row.computation else {
            panic!("captured state retained");
        };
        state.fixture_byte_formula()
    };
    assert_eq!(
        lineage.retention.retained_bytes(),
        bytes,
        "a fork and nested fork share the single captured charge"
    );
    let prior = PriorComputationRecord {
        cell: Arc::clone(&captured),
    };
    let next_bytes = sealed_run_for_lineage_test().state.fixture_byte_formula();
    let parent_next = || {
        let mut sealed = sealed_run_for_lineage_test();
        let row = captured.get().unwrap().mutable.lock().unwrap();
        let RecordedComputation::Retained(state) = &row.computation else {
            panic!("captured state retained");
        };
        sealed.cloned_from = Some(Arc::clone(state));
        sealed
    };
    let successor = lineage.retain_computation(
        parent_next(),
        Some(&prior),
        Some(&captured.get().unwrap().settlement_identity),
        lineage.prepay_computation_fork_scan_for_test(),
    );
    assert_eq!(
        lineage.retention.retained_bytes(),
        bytes + next_bytes,
        "a pinned parent's successor reserves a distinct full state"
    );
    assert!(matches!(
        captured.get().unwrap().mutable.lock().unwrap().computation,
        RecordedComputation::Retained(_)
    ));
    drop(successor);
    assert_eq!(lineage.retention.retained_bytes(), bytes);
    let successor = lineage.retain_computation(
        parent_next(),
        Some(&prior),
        Some(&captured.get().unwrap().settlement_identity),
        before_forks,
    );
    assert_eq!(
        lineage.retention.retained_bytes(),
        bytes + next_bytes,
        "forks created after prepayment preserve custody without an unbudgeted scan"
    );
    assert!(matches!(
        captured.get().unwrap().mutable.lock().unwrap().computation,
        RecordedComputation::Retained(_)
    ));
    drop(successor);
    assert_eq!(lineage.retention.retained_bytes(), bytes);
    let mut denied =
        InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: 0,
            maximum_preparation_bytes: 4096,
        });
    assert!(
        lineage.prepay_computation_fork_scan(&mut denied).is_err(),
        "pin-scan work is admitted before publication"
    );
    drop(prior);
    lineage.release_occurrence(parent.occurrence);
    assert!(!lineage.by_source[&source].contains_key(&child.lifecycle_incarnation()));
    assert!(!lineage.by_source[&source].contains_key(&nested.lifecycle_incarnation()));
    assert!(
        Arc::ptr_eq(&select(&lineage, &child), &captured),
        "a child with no local record selects the deleted ancestor's exact cell"
    );
    assert!(
        Arc::ptr_eq(&select(&lineage, &nested), &captured),
        "nested no-local inheritance still selects the same deleted ancestor cell"
    );
    assert_eq!(lineage.retention.retained_bytes(), bytes);
    restored(&mut lineage, &child);
    let local = select(&lineage, &child);
    assert!(!Arc::ptr_eq(&local, &captured));
    assert!(
        matches!(
            local.get().unwrap().mutable.lock().unwrap().computation,
            RecordedComputation::Absent(PriorAbsence::Restored)
        ),
        "local typed absence stops ancestor reuse"
    );
    assert!(Arc::ptr_eq(&select(&lineage, &nested), &local));
    assert!(
        lineage.by_source[&source].contains_key(&parent.occurrence),
        "live descendants preserve ancestor history"
    );
    lineage.release_occurrence(child.lifecycle_incarnation());
    lineage.release_occurrence(nested.lifecycle_incarnation());
    assert!(lineage.by_source.is_empty());
    assert_eq!(
        lineage.retention.retained_bytes(),
        bytes,
        "an independently selected prior retains its state after lineage deletion"
    );
    drop(local);
    drop(captured);
    assert_eq!(
        lineage.retention.retained_bytes(),
        0,
        "final holder releases the captured state's one ticket"
    );
}
