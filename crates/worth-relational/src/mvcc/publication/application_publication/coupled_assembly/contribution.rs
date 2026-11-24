//! Recorded applied participants, projected from the one Native group owner.
use super::super::RequiredOutputPublicationMeaning;
use super::producer_posture::RelationalCoupledProducerPosture;
use super::retained_assembly::{CoupledAuthoredStagePart, CoupledPublicationPart};
use crate::identity::data::EntityId;
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

pub(super) fn authored_stage(
    part: &CoupledAuthoredStagePart,
    source: EntityId,
) -> Option<RelationalCoupledContribution<'_>> {
    (part.source == source).then_some(RelationalCoupledContribution {
        records: &part.records,
        publication_required: false,
        ordinary: Some((&part.roles, &part.output_ids)),
    })
}

pub(super) fn produced<'a>(
    parts: &'a [CoupledPublicationPart],
    producer: &str,
    meaning: &RequiredOutputPublicationMeaning,
) -> Option<RelationalCoupledContribution<'a>> {
    parts.iter().find_map(|part| {
        let (actual, _) = part.publication.lineage_attribution()?;
        let actual_meaning = part.publication.producing_meaning()?;
        (actual == producer && actual_meaning == meaning).then_some(RelationalCoupledContribution {
            records: &part.records,
            publication_required: part.publication.owned().is_some(),
            ordinary: match (&part.publication, &part.ordinary_ids) {
                (RelationalCoupledProducerPosture::Ordinary { roles, .. }, Some(ids)) => {
                    Some((roles.as_slice(), ids))
                }
                _ => None,
            },
        })
    })
}

pub(super) fn route(part: &CoupledPublicationPart) -> (&str, &RequiredOutputPublicationMeaning) {
    let (producer, _) = part
        .publication
        .lineage_attribution()
        .expect("Native admitted a producer participant");
    let meaning = part
        .publication
        .producing_meaning()
        .expect("Native admitted producing origin");
    (producer, meaning)
}
