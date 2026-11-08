//! Recorded selectors compare with a genuine Native origin; they issue no custody.
use super::super::{RequiredOutputPublicationMeaning, RequiredOutputRolePosture};
use super::*;
use crate::{
    execution::request_stage::checkpoint,
    identity::data::{EntityId, KindId},
    transactions::data::TransactionCommitError,
};
#[derive(Clone, Copy)]
pub struct RecordedRequiredOutputRole<'a> {
    pub role: &'a str,
    pub entity_name: &'a str,
    pub kind: KindId,
    pub posture: RequiredOutputRolePosture,
    /// Unmaterialized targets have no recorded identity and always refuse.
    pub entity: Option<EntityId>,
}
impl RelationalRequiredOutputPublication {
    /// Descriptive equality grants no currentness, publication, or succession authority.
    pub fn matches_recorded_description<'a>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        producer: &str,
        dependency: &[u8; 32],
        meaning: &RequiredOutputPublicationMeaning,
        roles: impl ExactSizeIterator<Item = RecordedRequiredOutputRole<'a>> + Clone,
    ) -> Result<bool, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                charge(context, &self.origin, producer, meaning, roles.len())?;
                require(context, &self.origin, producer, dependency, meaning, roles)
            })
            .map_err(super::execution_stop)?
            .0
            .map(|answer| answer.is_ok())
    }
}
pub(super) fn require<'a>(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    origin: &crate::history::data::CanonicalRequiredOutputOrigin,
    producer: &str,
    dependency: &[u8; 32],
    meaning: &RequiredOutputPublicationMeaning,
    roles: impl ExactSizeIterator<Item = RecordedRequiredOutputRole<'a>> + Clone,
) -> Result<Result<(), EntityId>, TransactionCommitError> {
    if origin.producer != producer
        || &origin.dependency_identity != dependency
        || &origin.meaning != meaning
        || roles.len() != origin.members.len()
    {
        return Ok(Err(meaning.source_scope));
    }
    for (index, role) in roles.clone().enumerate() {
        checkpoint(
            context,
            index
                .saturating_add(origin.members.len())
                .saturating_add(role.role.len())
                .saturating_add(role.entity_name.len())
                .saturating_add(1) as u64,
        )
        .map_err(super::execution_stop)?;
        let Some(entity) = role.entity else {
            return Ok(Err(meaning.source_scope));
        };
        if roles
            .clone()
            .take(index)
            .any(|prior| prior.entity == Some(entity))
            || !origin.members.iter().any(|member| {
                member.id == entity
                    && member.role == role.role
                    && member.entity_name == role.entity_name
                    && member.kind == role.kind
                    && member.posture == role.posture
            })
        {
            return Ok(Err(entity));
        }
    }
    Ok(Ok(()))
}
pub(super) fn charge(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    origin: &crate::history::data::CanonicalRequiredOutputOrigin,
    producer: &str,
    meaning: &RequiredOutputPublicationMeaning,
    role_count: usize,
) -> Result<(), TransactionCommitError> {
    checkpoint(
        context,
        producer
            .len()
            .saturating_add(origin.producer.len())
            .saturating_add(meaning.family.len())
            .saturating_add(origin.meaning.family.len())
            .saturating_add(role_count)
            .saturating_add(96) as u64,
    )
    .map_err(super::execution_stop)?;
    Ok(())
}
