//! Two genuine producer routes share one actual coupled publication.
use super::super::super::fixture::{basis, locator, SOURCE_VALUE};
use super::super::fixture::{ProducerPublication, RelatedWorld};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::mvcc::{
    RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledProducerAssembly,
    RelationalCoupledAssemblyFailure, RelationalCoupledProducerPosture,
    RelationalRequiredOutputPublication, RelationalTransactionIntent, RequiredOutputPublication,
    RequiredOutputRolePosture,
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
            RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Preserve {
                admission,
                predecessor: self.predecessor.clone(),
                meaning: self.predecessor.origin.meaning.clone(),
                roles: crate::mvcc::native_roles_fixture(
                    &self.outputs.map(|id| (id, KindId(3))),
                    RequiredOutputRolePosture::Preserve,
                ),
            }),
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
