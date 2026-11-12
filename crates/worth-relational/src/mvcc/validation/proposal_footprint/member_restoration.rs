//! Logical member keys select correspondence only inside one validated draft.
use super::relations::stop;
use super::ValidatedRelationalProposal;
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::TransactionCommitError;
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, ContractValidatedAspectValueView,
};

/// Historical selectors grant no mutation or publication authority.
pub struct RecordedMemberRestoration<'a> {
    pub original: EntityId,
    pub kind: KindId,
    pub owner: (KindId, EntityId),
    pub key: (&'a AspectFieldLocator, &'a AspectValue),
    pub fields: &'a [(&'a AspectFieldLocator, &'a AspectValue)],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordedMemberRestorationRefusal {
    Incomplete(EntityId),
    Repeated(EntityId),
    Extra(EntityId),
    Foreign(EntityId),
    FieldMismatch(EntityId),
}
impl ValidatedRelationalProposal {
    /// Called once per actual entity birth by the relation footprint owner.
    /// Work is bounded by the declared restored members and their retained fields.
    /// A birth of a declared member kind that no recorded member claims is extra,
    /// also when the source recorded no member of that kind.
    pub(super) fn project_restored_member(
        &self,
        actual: EntityId,
        kind: KindId,
        members: &[RecordedMemberRestoration<'_>],
        member_kinds: &[KindId],
        restored: &mut Vec<(EntityId, EntityId)>,
        refusal: &mut Option<RecordedMemberRestorationRefusal>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        use crate::execution::request_stage::checkpoint;
        let state = crate::validation::engine::state_view::InvariantStateView::new(
            &self.state.proposed_mutation.state,
            self.state.evidence.proposed_version(),
        );
        let mut in_family = false;
        for member in members {
            checkpoint(context, 1).map_err(stop)?;
            if kind != member.kind {
                continue;
            }
            in_family = true;
            checkpoint(
                context,
                member.key.1.owned_allocation_capacity_bytes() as u64 + 1,
            )
            .map_err(stop)?;
            let field = |locator: &AspectFieldLocator| {
                let aspect = state
                    .entity_aspect_state(actual)?
                    .get(locator.aspect().aspect_key())?;
                match aspect.view() {
                    ContractValidatedAspectValueView::Scalar(value) => Some(value),
                    ContractValidatedAspectValueView::Struct(fields) => {
                        fields.get(locator.field_path().fields().first()?)
                    }
                }
            };
            if field(member.key.0) != Some(member.key.1) {
                continue;
            }
            checkpoint(context, restored.len() as u64).map_err(stop)?;
            if restored.iter().any(|(old, _)| *old == member.original) {
                *refusal = Some(RecordedMemberRestorationRefusal::Repeated(member.original));
                return Ok(());
            }
            restored.push((member.original, actual));
            for (locator, expected) in member.fields {
                checkpoint(
                    context,
                    expected.owned_allocation_capacity_bytes() as u64 + 1,
                )
                .map_err(stop)?;
                if field(locator) != Some(*expected) {
                    *refusal = Some(RecordedMemberRestorationRefusal::FieldMismatch(
                        member.original,
                    ));
                }
            }
            return Ok(());
        }
        checkpoint(context, member_kinds.len() as u64).map_err(stop)?;
        if in_family || member_kinds.contains(&kind) {
            *refusal = Some(RecordedMemberRestorationRefusal::Extra(actual));
        }
        Ok(())
    }
}
