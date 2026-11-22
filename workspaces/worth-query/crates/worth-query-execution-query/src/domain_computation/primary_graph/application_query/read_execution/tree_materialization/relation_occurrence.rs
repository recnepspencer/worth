//! Physical membership follows its selected child occurrence through projection and ordering.
use worth_relational::facade::identity::{EntityId, RelationId};

use super::{
    projection_denial, ResultTreeWork, WorthQueryApplicationProjectionNode,
    WorthQueryApplicationReadExecutionDenial,
};

#[derive(Clone, Copy)]
pub struct RelationProjectionTarget {
    pub entity: EntityId,
    pub relation: RelationId,
}

pub fn attach_occurrences(
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
