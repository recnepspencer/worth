use super::*;
use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    transactions::{
        DeleteEntityIntent, EntityMutationIntent, MutationIntent, RevalidateEntityIntent,
        WorkerIntentBatch,
    },
};

#[test]
fn inverse_cannot_retire_source_instance_but_may_change_subject_or_revalidate() {
    let instance = EntityId::new(PartitionId::main(), 1, 1);
    let subject = EntityId::new(PartitionId::main(), 2, 1);
    let retire = |entity_id| {
        WorkerIntentBatch::new("corrective-candidate").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        ))
    };
    assert_eq!(
        preserve_instance(&retire(instance), instance)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch
    );
    preserve_instance(&retire(subject), instance).unwrap();
    let unchanged = WorkerIntentBatch::new("ordinary-invariant-demand").push(
        MutationIntent::Entity(EntityMutationIntent::Revalidate(RevalidateEntityIntent {
            entity_id: instance,
        })),
    );
    preserve_instance(&unchanged, instance).unwrap();
}
