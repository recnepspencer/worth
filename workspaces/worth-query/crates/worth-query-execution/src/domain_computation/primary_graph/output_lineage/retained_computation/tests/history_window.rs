//! A superseded fork-pinned state refunds only outside retained history.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, publish_relational_mutation, AccountLabel,
    AccountStatus,
};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationSource, SignalBranchCreationPlan,
};

#[test]
fn a_superseded_pinned_parent_stays_charged_until_it_leaves_the_history_window() {
    let world = installed_authorization_world(true);
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let parent = product.observation();
    let intent = ProductBranchCreationIntent::from_source(
        "history-custody-child",
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
            &product,
            None,
            intent,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap()
    else {
        panic!("the real fork performs");
    };
    let account = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(account),
        output_binding: TypeId::of::<()>(),
    };
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage
        .retention
        .install(1 << 30, std::num::NonZeroUsize::new(2).unwrap());
    lineage.register_fork(parent, &child);
    let record =
        |lineage: &mut WorthQueryApplicationOutputLineage,
         observation: &worth_runtime_world::facade::ProductBranchObservation| {
            lineage.record_restoration(source.output_binding, source.runtime_authority, source.schema.clone(),
            source.scope, observation,
            Arc::new(super::super::super::WorthQueryApplicationOutputCorrespondence::default()),
            super::super::super::RecordedSourceIdentity::Checkpoint(
                crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x72; 32])),
            [0x71; 32], None, [0x73; 32],
            super::super::super::ComputationSourceEvidence::for_test(false).retain_facts(Arc::from([])), None, None);
        };
    record(&mut lineage, parent);
    let run = sealed_run_for_lineage_test();
    let bytes = run.state.fixture_byte_formula();
    let retained = lineage.retain_computation(
        run,
        None,
        None,
        lineage.prepay_computation_fork_scan_for_test(),
    );
    let old_generation = parent.reference_generation().get();
    let occurrence = parent.lifecycle_incarnation();
    lineage.by_source[&source][&occurrence][&old_generation][0]
        .get()
        .unwrap()
        .mutable
        .lock()
        .unwrap()
        .computation = retained;
    // No selected cell, state handle or observer artificially pins this row.
    let old_cell = Arc::downgrade(&lineage.by_source[&source][&occurrence][&old_generation][0]);
    let label = AccountLabel::reference();
    let graph = world.application.runtime.primary_graph().unwrap();
    let field = graph
        .layout()
        .field_locator(label.entity(), label.aspect(), label.field())
        .unwrap()
        .clone();
    for (index, value) in ["superseded", "outside-window"].into_iter().enumerate() {
        publish_relational_mutation(
            &world,
            WorkerIntentBatch::new("history-custody-window").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: account,
                    fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                        field.clone(),
                        AspectValue::String(InternedString::Raw(value.to_owned())),
                    )])),
                }),
            )),
        );
        let next = world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .unwrap();
        record(&mut lineage, next.observation());
        if index == 0 {
            assert_eq!(
                lineage.retention.retained_bytes(),
                bytes,
                "superseded while fork-pinned"
            );
            lineage.release_occurrence(child.lifecycle_incarnation());
            assert!(lineage.origins.is_empty());
            assert_eq!(
                lineage.retention.retained_bytes(),
                bytes,
                "last descendant deletion keeps in-window state"
            );
        }
        lineage
            .retire_unselected_generations(
                &source,
                occurrence,
                &mut InvalidationEditAdmission::new(
                    worth_relational::facade::mvcc::CompanionPreflightBudget {
                        maximum_work_visits: 1_000_000,
                        maximum_preparation_bytes: 1_000_000,
                    },
                ),
            )
            .unwrap();
        if index == 0 {
            assert!(old_cell.upgrade().is_some());
            assert_eq!(
                lineage.retention.retained_bytes(),
                bytes,
                "still inside the two-generation window"
            );
        } else {
            assert!(old_cell.upgrade().is_none());
            assert_eq!(
                lineage.retention.retained_bytes(),
                0,
                "outside the window refunds the final state ticket"
            );
        }
    }
}
