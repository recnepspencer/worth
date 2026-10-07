use std::{any::TypeId, sync::Arc};

use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationSource, SignalBranchCreationPlan,
};

use super::{checkpoint_identity, RestoredOutputBinding};
use crate::domain_computation::{
    authorization::WorthQueryOperationScopeBinding,
    primary_graph::output_lineage::{
        invalidation::InvalidationEditAdmission, ProductCoordinate, SemanticSource,
        WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
    },
};

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 1024 * 1024,
    })
}

#[test]
fn cutoff_candidate_pins_the_selected_ancestor_partition_and_denies_before_pin() {
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
        output_binding: TypeId::of::<RestoredOutputBinding>(),
    };
    let target_partition = [0x11; 32];
    let sibling_partition = [0x22; 32];
    let target = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.record_recovered_prior_output(
        source.output_binding,
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
        source.output_binding,
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

    let mut funded = admission(100_000);
    let selected = lineage
        .prior_input_cutoff_candidate::<RestoredOutputBinding>(
            &scope,
            &child,
            target_partition,
            &mut funded,
        )
        .expect("the selected ancestor lookup is funded")
        .expect("the matching parent partition is retained");
    assert!(Arc::ptr_eq(
        selected.settlement_identity(),
        selected_identity
    ));
    assert!(
        selected.prepared_input_key().is_none(),
        "restored rows cannot cut off"
    );

    let required = funded.charged_work();
    assert!(required > 1);
    let pins_before_denial = Arc::strong_count(selected_cell);
    let mut short = admission(required - 1);
    assert!(matches!(
        lineage.prior_input_cutoff_candidate::<RestoredOutputBinding>(
            &scope,
            &child,
            target_partition,
            &mut short,
        ),
        Err(CompanionPreflightStop::WorkExhausted { .. })
    ));
    assert_eq!(Arc::strong_count(selected_cell), pins_before_denial);

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
            &source,
            ProductCoordinate {
                occurrence: parent.lifecycle_incarnation(),
                generation: next_generation,
            },
            target_partition,
            &mut admission(100_000),
        )
        .expect("the newer empty slot is a bounded lookup");
    assert_eq!(prior, Some((parent.reference_generation().get(), 0)));
}
