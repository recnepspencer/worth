//! Recorded applied participants, projected from the one Native group owner.
use crate::mvcc::publication::application_publication::output_meaning::{
    RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
/// Actual participant records and whether Native admitted lifetime publication custody.
pub struct RelationalCoupledContribution<'a> {
    pub(super) records: &'a [crate::transactions::data::primitives::RecordRef],
    pub(super) publication_required: bool,
    pub(super) ordinary: Option<(
        &'a [RequiredOutputRoleTarget],
        &'a worth_execution::ExecutionArray<crate::identity::data::EntityId>,
    )>,
}
impl RelationalCoupledContribution<'_> {
    pub fn records(&self) -> &[crate::transactions::data::primitives::RecordRef] {
        self.records
    }
    pub fn publication_required(&self) -> bool {
        self.publication_required
    }
    /// Descriptive roles issued by the actual ordinary participant's allocation owner.
    pub fn ordinary_output_members(
        &self,
    ) -> Option<
        impl ExactSizeIterator<
            Item = (
                &str,
                &str,
                RequiredOutputRolePosture,
                crate::identity::data::EntityId,
            ),
        >,
    > {
        self.ordinary.map(|(roles, ids)| {
            roles.iter().zip(ids.elements()).map(|(role, id)| {
                (
                    role.role.as_str(),
                    role.entity_name.as_str(),
                    role.posture,
                    *id,
                )
            })
        })
    }
}
