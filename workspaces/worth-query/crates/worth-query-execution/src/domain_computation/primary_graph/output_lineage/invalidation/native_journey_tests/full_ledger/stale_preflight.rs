//! Losing the first fork cell cannot expose a losing candidate's expected head.
use super::*;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_relational::facade::history::BranchId;

#[test]
fn a_stale_preflight_after_an_unindexed_fork_winner_cannot_poison_later_writes() {
    let world = world_installing(|defaults| WorthQueryInvalidationResourceInstallation {
        maximum_retained_bytes: MAXIMUM_RETAINED_BYTES,
        ..defaults
    });
    let entity = resolved_account(&world, "open", &live_scope()).entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let field = AccountLabel::reference();
    let locator = graph
        .layout()
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let fork = BranchId("stale-preflight-fork".to_owned());
    let held = owner
        .resources
        .reserve_retained_capacity(
            MAXIMUM_RETAINED_BYTES - owner.resources.retained_capacity_bytes(),
        )
        .unwrap();
    handle.with_runtime_mut(|runtime| {
        let (_, source) = runtime
            .observe_fork_source(runtime.main_branch_identity().branch_id())
            .unwrap();
        runtime.fork_branch(fork.clone(), source).unwrap();
        let prepare = |runtime: &mut RelationalRuntime, value: &str| {
            let basis = runtime
                .admit_branch_basis(&runtime.branch_identity(&fork).unwrap())
                .unwrap();
            let mut tx = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            tx.push_batch(WorkerIntentBatch::new(value).push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        locator.clone(),
                        AspectValue::String(InternedString::Raw(value.to_owned())),
                    )])),
                }),
            )))
            .unwrap();
            runtime.prepare_branch_transaction(tx).unwrap()
        };
        let w1 = prepare(runtime, "winner");
        let w2 = prepare(runtime, "stale-loser");
        let RelationalPublicationOutcome::Performed(performed) =
            runtime.publication_port().compare_and_publish(w1)
        else {
            panic!("the full ledger cannot refuse W1");
        };
        let committed = runtime.settle_performed_publication(performed).unwrap();
        release_test_commit_snapshot(runtime, &committed);
        drop(held);
        let stale = runtime.publication_port().compare_and_publish(w2);
        assert!(
            matches!(stale, RelationalPublicationOutcome::Stale(_)),
            "W2 must be stale: {stale:?}"
        );
        let later = prepare(runtime, "later-writer");
        let outcome = runtime.publication_port().compare_and_publish(later);
        let RelationalPublicationOutcome::Performed(performed) = outcome else {
            panic!(
                "a stale candidate cannot poison the lookup or defer a later writer: {outcome:?}"
            );
        };
        let committed = runtime.settle_performed_publication(performed).unwrap();
        release_test_commit_snapshot(runtime, &committed);
        assert!(owner.resources.retained_capacity_bytes() <= MAXIMUM_RETAINED_BYTES);
    });
}
