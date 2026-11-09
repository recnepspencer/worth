//! Native's exact set comparison, independent of the storage traversal.
use super::*;
use crate::transactions::data::EntityReference;
use std::collections::BTreeSet;

pub(super) struct OwnedMemberRoster {
    anchor: EntityId,
    expected: BTreeSet<EntityId>,
    supplied: BTreeSet<EntityId>,
}
impl OwnedMemberRoster {
    pub(super) fn new(anchor: EntityId) -> Self {
        Self {
            anchor,
            expected: BTreeSet::new(),
            supplied: BTreeSet::new(),
        }
    }
    pub(super) fn observe_member(
        &mut self,
        member: EntityId,
        observed_kind: Option<KindId>,
        expected_kind: KindId,
    ) -> Result<(), TransactionCommitError> {
        if observed_kind != Some(expected_kind) {
            return Err(refuse(Cause::DanglingOwnedMember, member));
        }
        if !self.expected.insert(member) {
            return Err(refuse(Cause::RepeatedOwnedMember, member));
        }
        Ok(())
    }
    pub(super) fn supply(
        &mut self,
        role: &RequiredOutputRoleTarget,
    ) -> Result<(), TransactionCommitError> {
        let EntityReference::Existing(id) = role.target else {
            return Err(refuse(Cause::ExtraOwnedMember, self.anchor));
        };
        if !self.supplied.insert(id) {
            return Err(refuse(Cause::RepeatedOwnedMember, id));
        }
        if !self.expected.contains(&id) {
            return Err(refuse(Cause::ForeignOwnedMember, id));
        }
        Ok(())
    }
    pub(super) fn finish(self) -> Result<(), TransactionCommitError> {
        if let Some(id) = self.expected.difference(&self.supplied).next() {
            return Err(refuse(Cause::MissingOwnedMember, *id));
        }
        Ok(())
    }
}
fn refuse(cause: Cause, id: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        cause,
        RecordRef::Entity(id),
    ))
}
#[cfg(test)]
mod tests;

pub(super) fn owner_metadata(
    id: crate::identity::data::RelationId,
    metadata: Option<crate::validation::engine::state_view::VisibleRelationMetadata>,
) -> Result<crate::validation::engine::state_view::VisibleRelationMetadata, TransactionCommitError>
{
    metadata.ok_or_else(|| {
        TransactionCommitError::publication(PublicationError::authored_retirement(
            Cause::DanglingOwnedMember,
            RecordRef::Relation(id),
        ))
    })
}
