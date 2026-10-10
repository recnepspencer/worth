//! A restored root read before an unindexed first fork publication cannot mint at that old basis.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::{
    ConsumedOutputEvidence, ConsumedOutputVerification,
};
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_relational::facade::history::BranchId;

pub(super) fn restored_root_consumption_and_later_writes() {
    let world = world_installing(|defaults| WorthQueryInvalidationResourceInstallation {
        maximum_retained_bytes: MAXIMUM_RETAINED_BYTES,
        ..defaults
    });
    let entity = resolved_account(&world, "open", &live_scope()).entity_id();
    let unrelated = resolved_account(&world, "unrelated", &live_scope()).entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let label = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label.entity(), label.aspect(), label.field())
        .unwrap()
        .clone();
    let status = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status.entity(), status.aspect(), status.field())
        .unwrap()
        .clone();
    let (_, product, _) = world.selected_product().into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let fork = BranchId("full-ledger-new-branch".to_owned());
    let (old_handle, old, root) = handle.with_runtime_mut(|runtime| {
        let (_, source) = runtime.observe_fork_source(runtime.main_branch_identity().branch_id()).unwrap();
        runtime.fork_branch(fork.clone(), source).unwrap();
        let basis = runtime.admit_branch_basis(&runtime.branch_identity(&fork).unwrap()).unwrap();
        let old_handle = runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap();
        let old = Arc::new(runtime.read_truth().positioned_snapshot(&old_handle).unwrap());
        let source = SemanticSource { runtime_authority: world.application.runtime.authority_identity().as_u64(), schema: world.application.installed_schema.binding_identity(), scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity), output_binding: OutputBindingIdentity::declared("StatusOutput") };
        let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
        let facts = crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::from([field_fact(runtime, &old_handle, entity, status)])).for_comparison().unwrap();
        // Real Native source/output facts, read and fully compared before the fork's first write.
        let witness = super::super::request_recording::account_witness(&world, runtime, &old_handle, entity);
        let current = ConsumedOutputEvidence::verify_at_observation(&identity, &facts, &[], Some(crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason::CheckpointRestore), &witness, owner, runtime, &old_handle, &old, &mut owner.edit_admission()).unwrap();
        assert_eq!(current, ConsumedOutputVerification::Current);
        let root = ConsumedOutputEvidence::restored_with_witness_for_test(owner, identity, Arc::clone(facts.facts()), Arc::clone(&old), witness);
        (old_handle, old, root)
    });
    let held = owner
        .resources
        .reserve_retained_capacity(
            MAXIMUM_RETAINED_BYTES - owner.resources.retained_capacity_bytes(),
        )
        .unwrap();
    handle.with_runtime_mut(|runtime| {
        publish_on_fork(runtime, &fork, unrelated, label.clone(), "first-fork-write");
        assert!(owner.resources.retained_capacity_bytes() <= MAXIMUM_RETAINED_BYTES);
        assert!(matches!(
            owner
                .currentness(&old, root.identity(), &mut owner.edit_admission())
                .unwrap(),
            SourceSettlementCurrentness::FullVerificationRequired(_)
        ));
    });
    drop(held);
    // The production consumer precommit path re-establishes the restored root.
    // Its read basis is old; that basis must never select the minting head.
    root.establish_restored_before_commit(&handle.source_owner, &mut owner.edit_admission())
        .unwrap();
    handle.with_runtime_mut(|runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.branch_identity(&fork).unwrap())
            .unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let selected = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
        assert_ne!(old.root_id(), selected.root_id());
        assert_eq!(
            ConsumedOutputEvidence::verify_many_with_admission(
                std::slice::from_ref(&root),
                owner,
                runtime,
                &snapshot,
                &selected,
                &mut owner.edit_admission()
            )
            .unwrap(),
            ConsumedOutputVerification::Current,
            "the unrelated write did not change the restored root"
        );
        publish_on_fork(runtime, &fork, unrelated, label.clone(), "consumer-commit");
        publish_on_fork(
            runtime,
            &fork,
            unrelated,
            label.clone(),
            "later-fork-write-one",
        );
        publish_on_fork(runtime, &fork, unrelated, label, "later-fork-write-two");
        assert!(owner.resources.retained_capacity_bytes() <= MAXIMUM_RETAINED_BYTES);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        runtime.snapshots().release_snapshot(&old_handle).unwrap();
    });
}

fn publish_on_fork(
    runtime: &mut RelationalRuntime,
    fork: &BranchId,
    entity: EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let basis = runtime
        .admit_branch_basis(&runtime.branch_identity(fork).unwrap())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(value).push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        locator,
                        AspectValue::String(InternedString::Raw(value.to_owned())),
                    )])),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let outcome = runtime.publication_port().compare_and_publish(candidate);
    let RelationalPublicationOutcome::Performed(performed) = outcome else {
        panic!("{value} must publish, including the consumer and both later writes: {outcome:?}");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
