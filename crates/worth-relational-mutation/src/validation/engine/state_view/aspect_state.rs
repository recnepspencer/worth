use worth_foundational::facade::AuthoritativeRecordAspectState;

use worth_relational_contracts::internal_port::identity::data::record_ids::{EntityId, RelationId};

use super::InvariantStateView;

impl<'state> InvariantStateView<'state> {
    pub(crate) fn entity_aspect_state(
        &self,
        entity_id: EntityId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let locate = || self.selected.locate_entity_aspect(entity_id);
        let location = match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.entity_aspect(*basis, self.version_id(), entity_id, locate)
            }
            None => locate(),
        }?;
        self.selected.entity_aspect_at(entity_id, location)
    }

    pub(crate) fn relation_aspect_state(
        &self,
        relation_id: RelationId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let locate = || self.selected.locate_relation_aspect(relation_id);
        let location = match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.relation_aspect(*basis, self.version_id(), relation_id, locate)
            }
            None => locate(),
        }?;
        self.selected.relation_aspect_at(relation_id, location)
    }
}
