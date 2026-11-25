//! Closed performed native graph retirement precedes changed-codec consumers.
mod codec_disposition;
mod consequences;
mod fixture;
mod incoming_reference;
mod refusals;
mod settlement;
use super::fixture::{basis, capture, capture_with_source, protected_payload};
use crate::indexes::data::DerivedIndexEntries;
use crate::mvcc::RelationalOperationControl;
use fixture::*;

#[test]
fn closed_three_outputs_two_relations_retire_atomically_and_regenerate_ordered_v2() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution);
    assert_order(&world.runtime, &world.outputs, &world.relations);
    assert_values(
        &world.runtime,
        &world.outputs,
        [
            "codec1:nondefault-0",
            "codec1:nondefault-1",
            "codec1:nondefault-2",
        ],
    );
    let old_basis = basis(&world.runtime);
    let old_index = world
        .runtime
        .indexes
        .latest_generation(ORDER, Some(old_basis.descriptor().branch_id()))
        .unwrap();
    let source = protected_payload(&world.runtime, world.authored);
    assert!(!source.is_empty());
    let payloads = world
        .outputs
        .map(|id| protected_payload(&world.runtime, id));
    let edges = world
        .relations
        .map(|id| relation_payload(&world.runtime, id));
    let checkpoint = capture(execution, &world.runtime);
    let checkpoint_bytes = checkpoint.bytes().to_vec();
    let (mut target, mut recovered, receipt) = recover(execution, &world);
    assert!(target
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .is_err());
    let candidate = candidate(
        execution,
        &mut target,
        &mut recovered,
        world.activation,
        RelationalOperationControl::uninterrupted(),
        None,
    )
    .unwrap();
    let committed = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap();
    let (commit, recovered) = committed.into_parts();
    let target_receipt = target
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let current = basis(&target);
    for (position, id) in world.outputs.iter().enumerate() {
        assert!(recovered.codec_transition_retires(*id));
        assert_eq!(protected_payload(&target, *id), payloads[position]);
        assert!(!current
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .unwrap()
            .entity_arena
            .get(id)
            .unwrap()
            .is_live());
        assert!(old_basis
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .unwrap()
            .entity_arena
            .get(id)
            .unwrap()
            .is_live());
    }
    for (position, id) in world.relations.iter().enumerate() {
        assert_eq!(relation_payload(&target, *id), edges[position]);
        assert!(!current
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .unwrap()
            .relation_arena
            .get(id)
            .unwrap()
            .is_live());
        assert!(old_basis
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .unwrap()
            .relation_arena
            .get(id)
            .unwrap()
            .is_live());
    }
    assert_eq!(protected_payload(&target, world.authored), source);
    let retired_index = target
        .indexes
        .latest_generation(ORDER, Some(current.descriptor().branch_id()))
        .unwrap();
    assert_eq!(retired_index.source_commit_id, commit.commit_id);
    assert_ne!(retired_index.generation_id, old_index.generation_id);
    assert!(
        matches!(&retired_index.entries, DerivedIndexEntries::RelatedEntityOrdering(entries) if entries.is_empty())
    );
    recovered.admit_basis(current).unwrap();
    codec_disposition::cold_retired_route(execution, &target);
    let (outputs, relations) = produce(execution, &target, &target_receipt, 2);
    for id in outputs {
        assert!(!world.outputs.contains(&id));
    }
    super::fixture::build_index(execution, &target);
    assert_order(&target, &outputs, &relations);
    assert_values(
        &target,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    let v2_payloads = outputs.map(|id| protected_payload(&target, id));
    assert_ne!(v2_payloads, payloads);
    let (saved, saved_source) = capture_with_source(execution, &target);
    let (mut reopened, receipt) = installed(2);
    reopened
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &saved, saved_source)
        .unwrap();
    reopened
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_order(&reopened, &outputs, &relations);
    assert_values(
        &reopened,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    assert_eq!(protected_payload(&reopened, world.authored), source);
    for (position, id) in world.outputs.iter().enumerate() {
        assert_eq!(protected_payload(&reopened, *id), payloads[position]);
    }
    for (position, id) in world.relations.iter().enumerate() {
        assert_eq!(relation_payload(&reopened, *id), edges[position]);
    }
    assert_eq!(checkpoint.bytes(), checkpoint_bytes);
    assert_order(&world.runtime, &world.outputs, &world.relations);
    assert_eq!(
        old_index.entries,
        world
            .runtime
            .indexes
            .latest_generation(ORDER, Some(old_basis.descriptor().branch_id()))
            .unwrap()
            .entries
    );
}

mod coupled_sibling {
    //! Two genuine producer routes share one actual coupled publication.
    use super::super::fixture::{basis, locator, SOURCE_VALUE};
    use super::fixture::{ProducerPublication, RelatedWorld};
    use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
    use crate::mvcc::{
        RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledProducerAssembly,
        RelationalCoupledAssemblyFailure, RelationalCoupledProducerPosture,
        RelationalRequiredOutputPublication, RelationalTransactionIntent,
        RequiredOutputPublication, RequiredOutputRolePosture,
    };
    use crate::transactions::data::*;
    use worth_execution::ExecutionRequest;
    use worth_foundational::facade::AspectValue;

    struct SiblingAssembly<'a> {
        owner: &'a worth_proof::InstalledApplicationPublicationOwner,
        predecessor: RelationalRequiredOutputPublication,
        outputs: [EntityId; 3],
        relations: [RelationId; 2],
        sibling: CreatedEntityRef,
        sibling_entity: std::cell::Cell<Option<EntityId>>,
    }
    impl RelationalAuthoredCoupledProducerAssembly for SiblingAssembly<'_> {
        fn assemble(
            &self,
            execution: &ExecutionRequest<'_, '_>,
            candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        ) -> Result<std::ops::ControlFlow<()>, RelationalCoupledAssemblyFailure> {
            let admission = self
                .owner
                .admit_required_output_succession(
                    self.owner
                        .admit_required_output("native.closed.producer", [19; 32]),
                    self.predecessor.authority(),
                )
                .unwrap();
            let mut batch = WorkerIntentBatch::new("preserve the complete first group");
            for entity_id in self.outputs {
                batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
                    RevalidateEntityIntent { entity_id },
                )));
            }
            for relation_id in self.relations {
                batch = batch.push(MutationIntent::Relation(
                    RelationMutationIntent::Revalidate(RevalidateRelationIntent { relation_id }),
                ));
            }
            candidate.append_publication(
                execution,
                RelationalCoupledProducerPosture::Publication(
                    RequiredOutputPublication::Preserve {
                        admission,
                        predecessor: self.predecessor.clone(),
                        meaning: self.predecessor.origin.meaning.clone(),
                        roles: crate::mvcc::native_roles_fixture(
                            &self.outputs.map(|id| (id, KindId(3))),
                            RequiredOutputRolePosture::Preserve,
                        ),
                    },
                ),
                &crate::content::RelationalContentAttachments::default(),
                vec![batch],
            )?;
            let publication = crate::mvcc::native_birth_fixture(
                self.owner
                    .admit_required_output("native.sibling.producer", [23; 32]),
                vec![self.sibling.clone()],
            );
            let batch = WorkerIntentBatch::new("create the independent sibling group").push(
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: self.sibling.partition_id,
                    kind_id: self.sibling.kind_id,
                    client_key: self.sibling.client_key.clone(),
                    fields: AspectFieldPatch::from_locator(
                        locator("derived"),
                        AspectValue::String("nondefault sibling output".into()),
                    ),
                })),
            );
            let meaning = publication
                .producing_meaning()
                .expect("actual sibling publication")
                .clone();
            candidate.append_publication(
                execution,
                RelationalCoupledProducerPosture::Publication(publication),
                &crate::content::RelationalContentAttachments::default(),
                vec![batch],
            )?;
            let entities = candidate
                .appended_output_entities(execution, "native.sibling.producer", &meaning)?
                .expect("Native appended the actual sibling route");
            assert_eq!(
                entities.len(),
                1,
                "one independently declared sibling output"
            );
            self.sibling_entity.set(Some(entities[0]));
            Ok(std::ops::ControlFlow::Continue(()))
        }
    }

    pub(super) fn publish(
        execution: &ExecutionRequest<'_, '_>,
        world: &RelatedWorld,
        publisher: &ProducerPublication,
    ) -> EntityId {
        let runtime = &world.runtime;
        let (owner, installed) = (&publisher.owner, &publisher.installed);
        let selected = basis(runtime);
        let predecessor = publisher.custody.clone();
        let owners = runtime
            .readmit_edit_coupled_owners(execution, installed, &selected, world.authored)
            .unwrap();
        let sibling = CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(3),
            client_key: crate::symbols::data::ClientKey::raw("independent sibling"),
        };
        let mut transaction = runtime
            .begin_branch_transaction(&selected, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("authentic authored prefix").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: world.authored,
                        fields: AspectFieldPatch::from_locator(
                            locator("source"),
                            AspectValue::String(SOURCE_VALUE.into()),
                        ),
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        let bound = transaction
            .bind_authored_coupled_producers(installed, owners, None)
            .unwrap();
        let assembly = SiblingAssembly {
            owner,
            predecessor,
            outputs: world.outputs,
            relations: world.relations,
            sibling,
            sibling_entity: std::cell::Cell::new(None),
        };
        let transaction = runtime
            .preparation_port()
            .prepare_authored_coupled_transaction(execution, bound, &assembly)
            .unwrap()
            .expect("both complete groups assemble synchronously");
        let committed = transaction.commit(execution, runtime).unwrap();
        let id = assembly
            .sibling_entity
            .get()
            .expect("Native issued the sibling identity");
        execution
            .run_native_with_checkpoint(|context| {
                let origins = committed
                    .envelope()
                    .validate_output_origins(context)
                    .unwrap();
                let first = origins
                    .output_origin_for_entity(world.outputs[0], context)
                    .unwrap();
                let second = origins.output_origin_for_entity(id, context).unwrap();
                assert_eq!(
                    first.commit, second.commit,
                    "two real producer groups share the sole canonical commit"
                );
                assert_eq!(first.commit, committed.commit);
                assert_eq!(first.meaning.family, "native.closed.producer");
                assert_eq!(second.meaning.family, "native.sibling.producer");
                assert!(!first.meaning.same_route(&second.meaning));
                assert_eq!(first.outputs.len(), world.outputs.len());
                assert_eq!(
                    first
                        .outputs
                        .iter()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>(),
                    world
                        .outputs
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>(),
                    "the complete independently issued first group is retained"
                );
                assert_eq!(first.relations.len(), 2);
                assert_eq!(second.outputs.as_slice(), &[id]);
            })
            .unwrap();
        runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
        id
    }
}
