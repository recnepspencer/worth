use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;

use worth_runtime_world::facade::ProductBranchCreationIntent;
use worth_runtime_world::facade::ProductBranchCreationPlans;
use worth_runtime_world::facade::RelationalBranchCreationPlan;
use worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome;
use worth_runtime_world::facade::RuntimeWorldCancellationSource;
use worth_runtime_world::facade::SignalBranchCreationPlan;

use super::checkpoint_identity;
use super::RestoredOutputBinding;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use crate::domain_computation::primary_graph::output_lineage::SemanticSource;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputCorrespondence;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;

#[test]
fn cutoff_candidate_pins_the_selected_ancestor_partition() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("a real product occurrence is live");
    let parent = product.observation();
    let intent = ProductBranchCreationIntent::from_source(
        "cutoff-ancestor",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .expect("the fork intent is valid");
    let RuntimeWorldBranchCreationOutcome::Performed(child) = world
        .application
        .product_runtime()
        .create_product_branch(
            execution,
            &product,
            None,
            intent,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the product owner admits the fork")
    else {
        panic!("the product owner must create the fork");
    };

    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let scope_entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let scope = WorthQueryOperationScopeBinding::axis_probe_scope(
        runtime_authority,
        schema.clone(),
        "cutoff-selection",
        scope_entity.partition_value(),
        scope_entity.local_slot_value(),
        scope_entity.generation_value(),
        scope_entity.partition_value(),
        scope_entity.local_slot_value(),
        scope_entity.generation_value(),
    );
    let source = SemanticSource {
        runtime_authority,
        schema: schema.clone(),
        scope: scope.scope(),
        output_binding: OutputBindingIdentity::declared("RestoredOutputBinding"),
    };
    let target_partition = [0x11; 32];
    let sibling_partition = [0x22; 32];
    let target = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    lineage.record_recovered_prior_output(
        lineage.binding_type(&source.output_binding),
        runtime_authority,
        schema.clone(),
        source.scope,
        parent.lifecycle_incarnation(),
        parent.reference_generation().get(),
        Arc::clone(&target),
        checkpoint_identity([0x31; 32]),
        target_partition,
        None,
        [0x41; 32],
        None,
    );
    lineage.record_recovered_prior_output(
        lineage.binding_type(&source.output_binding),
        runtime_authority,
        schema,
        source.scope,
        parent.lifecycle_incarnation(),
        parent.reference_generation().get(),
        sibling,
        checkpoint_identity([0x32; 32]),
        sibling_partition,
        None,
        [0x42; 32],
        None,
    );
    lineage.register_fork(parent, &child);
    let selected_cell = &lineage.by_source[&source][&parent.lifecycle_incarnation()]
        [&parent.reference_generation().get()][0];
    let selected_identity = &selected_cell.get().unwrap().settlement_identity;
    assert!(matches!(&selected_cell.get().unwrap().mutable.lock().unwrap().computation,
        super::super::retained_computation::RecordedComputation::Absent(
            crate::domain_computation::primary_graph::application_contribution::PriorAbsence::Restored)));

    let selected = lineage
        .prior_input_cutoff_candidate::<RestoredOutputBinding>(
            &scope,
            &child,
            target_partition,
            execution,
        )
        .expect("the selected ancestor lookup selects exact custody")
        .expect("the matching parent partition is retained");
    assert!(Arc::ptr_eq(
        selected.settlement_identity(),
        selected_identity
    ));
    assert!(
        selected.prepared_input_key().is_none(),
        "restored rows cannot cut off"
    );

    let next_generation = parent.reference_generation().get() + 1;
    let _unpublished = lineage.partition_index.insert_vacancy(
        source.clone(),
        ProductCoordinate {
            occurrence: parent.lifecycle_incarnation(),
            generation: next_generation,
        },
        Some(target_partition),
    );
    let prior = lineage
        .partition_index
        .latest_admitted(
            execution,
            &source,
            ProductCoordinate {
                occurrence: parent.lifecycle_incarnation(),
                generation: next_generation,
            },
            target_partition,
        )
        .expect("the newer empty slot is a bounded lookup");
    assert_eq!(prior, Some((parent.reference_generation().get(), 0)));
}
