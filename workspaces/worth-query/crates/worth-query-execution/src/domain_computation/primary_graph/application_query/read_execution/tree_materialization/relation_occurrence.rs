//! Physical membership follows its selected child occurrence through projection and ordering.
use worth_relational::facade::identity::{EntityId, RelationId};

use super::{
    projection_denial, ResultTreeWork, WorthQueryApplicationProjectionNode,
    WorthQueryApplicationReadExecutionDenial,
};

#[derive(Clone, Copy)]
pub(super) struct RelationProjectionTarget {
    pub(super) entity: EntityId,
    pub(super) relation: RelationId,
}

pub(super) fn attach_occurrences(
    children: &mut [WorthQueryApplicationProjectionNode],
    targets: &[RelationProjectionTarget],
    subject: &str,
    work: &ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    if children.len() != targets.len() {
        return Err(projection_denial(subject));
    }
    for (child, target) in children.iter_mut().zip(targets) {
        work.checkpoint(subject)?;
        if child.entity_id() != target.entity || !child.attach_incoming_relation(target.relation) {
            return Err(projection_denial(subject));
        }
    }
    Ok(())
}
