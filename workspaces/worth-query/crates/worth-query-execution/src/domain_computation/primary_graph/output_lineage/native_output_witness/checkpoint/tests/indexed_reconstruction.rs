//! Many real native outputs reconstruct once, rather than scanning F per role.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountIdentity, AccountMembershipTag,
};
use worth_relational::facade::{
    symbols::ClientKey,
    transactions::{CreateIntent, CreatedEntityRef, EntitySpec},
};

#[test]
fn checkpoint_output_reconstruction_indexes_complete_original_facts_with_bounded_work() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let seed = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    handle.with_runtime_mut(|runtime| {
        let mut measured = Vec::new();
        for count in [32, 128] {
            let entities = create_outputs(runtime, layout, seed, count);
            let before = snapshot(runtime);
            let truth = runtime.read_truth();
            let mut facts = Vec::new();
            for entity in &entities {
                facts.push(Fact::Entity {
                    entity_id: *entity,
                    kind: truth
                        .exact_snapshot_live_entity_kind(&before, *entity)
                        .unwrap(),
                });
                for aspect in layout.native_output_aspects("Account") {
                    facts.push(Fact::SourceAspectRevision {
                        entity_id: *entity,
                        aspect: aspect.clone(),
                        native_revision: truth
                            .exact_snapshot_entity_aspect_version(&before, *entity, aspect)
                            .unwrap(),
                    });
                }
            }
            // Original ordering is not an index and identical duplicates remain
            // legal. Neither may change the admitted expectations.
            facts.reverse();
            facts.push(facts[0].clone());
            let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
                TypeId::of::<()>(),
                TypeId::of::<()>(),
                Default::default(),
                entities
                    .iter()
                    .enumerate()
                    .map(|(ordinal, entity)| WorthQueryCheckpointOutputRole {
                        role: format!("account-{ordinal}"),
                        posture: WorthQueryApplicationOutputPosture::Preserve,
                        entity_name: "Account".into(),
                        entity: *entity,
                    })
                    .collect(),
                |_| Some(TypeId::of::<()>()),
            )
            .unwrap();
            let bound = SealedNativeOutputWitness::checkpoint_comparison_work_bound(
                &correspondence,
                layout,
                &facts,
                &mut owner.edit_admission(),
            )
            .unwrap();
            let mut admission = owner.read_admission(1_000_000);
            let restored = SealedNativeOutputWitness::from_checkpoint_facts(
                &correspondence,
                layout,
                &facts,
                owner,
                &mut admission,
            )
            .unwrap()
            .unwrap();
            let reconstruction_work = usize::try_from(admission.charged_work()).unwrap();
            assert!(restored
                .get()
                .unwrap()
                .checkpoint_facts_current_in(runtime, &before, &facts, &mut admission)
                .unwrap());
            let work = admission.charged_work();
            assert!(
                work <= bound,
                "all reconstruction and comparison work is bounded"
            );
            measured.push(work);
            let mut exhausted = owner.read_admission(reconstruction_work - 1);
            assert!(matches!(
                SealedNativeOutputWitness::from_checkpoint_facts(
                    &correspondence,
                    layout,
                    &facts,
                    owner,
                    &mut exhausted
                ),
                Err(CompanionPreflightStop::WorkExhausted { .. })
            ));
            if count == 128 {
                let first = facts
                    .iter()
                    .find_map(|fact| match fact {
                        Fact::Entity { entity_id, kind } => Some((*entity_id, *kind)),
                        _ => None,
                    })
                    .unwrap();
                let mut conflicting = facts.clone();
                conflicting.push(Fact::Entity {
                    entity_id: first.0,
                    kind: worth_relational::facade::identity::KindId(first.1 .0 + 1),
                });
                assert!(
                    SealedNativeOutputWitness::from_checkpoint_facts(
                        &correspondence,
                        layout,
                        &conflicting,
                        owner,
                        &mut owner.edit_admission()
                    )
                    .unwrap()
                    .is_none(),
                    "a contradictory duplicate cannot be hidden by the index"
                );
                let mut unrelated_conflicts = facts.clone();
                let seed_kind = truth
                    .exact_snapshot_live_entity_kind(&before, seed)
                    .unwrap();
                unrelated_conflicts.extend([
                    Fact::Entity {
                        entity_id: seed,
                        kind: seed_kind,
                    },
                    Fact::Entity {
                        entity_id: seed,
                        kind: worth_relational::facade::identity::KindId(seed_kind.0 + 1),
                    },
                ]);
                let aspect = layout.native_output_aspects("Account").next().unwrap();
                for revision in [Some(0), Some(1)] {
                    unrelated_conflicts.push(Fact::SourceAspectRevision {
                        entity_id: seed,
                        aspect: aspect.clone(),
                        native_revision: revision,
                    });
                }
                assert!(
                    SealedNativeOutputWitness::from_checkpoint_facts(
                        &correspondence,
                        layout,
                        &unrelated_conflicts,
                        owner,
                        &mut owner.edit_admission(),
                    )
                    .unwrap()
                    .is_some(),
                    "conflicts outside output roles cannot change output expectations"
                );
                // The independent full source-currentness comparison still
                // rejects these invalid source facts; this only checks witness
                // reconstruction's original, output-scoped contract.
            }
            runtime.snapshots().release_snapshot(&before).unwrap();
        }
        assert!(
            measured[1] < measured[0] * 8,
            "four times the facts and roles must not cause quadratic reconstruction: {measured:?}"
        );
        eprintln!("checkpoint reconstruction 32/128 real output work: {measured:?}");
    });
}

fn create_outputs(
    runtime: &mut RelationalRuntime,
    layout: &WorthQueryPrimaryGraphLayout,
    seed: EntityId,
    count: usize,
) -> Vec<EntityId> {
    let before = snapshot(runtime);
    let kind = runtime
        .read_truth()
        .exact_snapshot_live_entity_kind(&before, seed)
        .unwrap();
    runtime.snapshots().release_snapshot(&before).unwrap();
    let references = (0..count)
        .map(|ordinal| CreatedEntityRef {
            partition_id: seed.partition_id,
            kind_id: kind,
            client_key: ClientKey::raw(format!("checkpoint-{count}-{ordinal}")),
        })
        .collect::<Vec<_>>();
    let mut batch = WorkerIntentBatch::new("checkpoint-native-output-roster");
    for reference in &references {
        let key = reference.client_key.as_raw_str().unwrap();
        let identity = AccountIdentity::reference();
        let status = AccountStatus::reference();
        let membership = AccountMembershipTag::reference();
        let label = AccountLabel::reference();
        let fields = [
            (identity.entity(), identity.aspect(), identity.field(), key),
            (status.entity(), status.aspect(), status.field(), key),
            (
                membership.entity(),
                membership.aspect(),
                membership.field(),
                "open",
            ),
            (
                label.entity(),
                label.aspect(),
                label.field(),
                "Original output",
            ),
        ]
        .into_iter()
        .map(|(entity, aspect, field, value)| {
            let planned = layout.field_locator(entity, aspect, field).unwrap();
            (
                AspectFieldLocator::new(
                    LocatorAuthority::Authoritative,
                    planned.aspect().aspect_key().clone(),
                    planned.field_path().clone(),
                ),
                AspectValue::String(value.into()),
            )
        })
        .collect::<BTreeMap<_, _>>();
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: reference.partition_id,
            kind_id: kind,
            client_key: reference.client_key.clone(),
            fields: AspectFieldPatch::from(fields),
        })));
    }
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("the real output roster must publish");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let entities = references
        .iter()
        .map(|reference| committed.created_entity(reference).unwrap())
        .collect();
    release_test_commit_snapshot(runtime, &committed);
    entities
}
