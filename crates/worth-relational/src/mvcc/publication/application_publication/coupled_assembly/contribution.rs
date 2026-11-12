//! Recorded applied participants, projected from the one Native group owner.
use super::*;
/// Actual participant records and whether Native admitted lifetime publication custody.
pub struct RelationalCoupledContribution<'a> {
    records: &'a [crate::transactions::data::RecordRef],
    publication_required: bool,
    ordinary: Option<(
        &'a [RequiredOutputRoleTarget],
        &'a worth_execution::ExecutionArray<crate::identity::data::EntityId>,
    )>,
}
impl RelationalCoupledContribution<'_> {
    pub fn records(&self) -> &[crate::transactions::data::RecordRef] {
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
impl crate::mvcc::ValidatedRelationalProposal {
    /// The exact applied contribution of a later authored stage. The stage's
    /// admitted source must match; descriptive ordinals cannot select a foreign
    /// contribution or grant any publication authority.
    pub fn authored_stage_contribution_records(
        &self,
        stage: usize,
        source: crate::identity::data::EntityId,
    ) -> Option<RelationalCoupledContribution<'_>> {
        let group =
            self.state.required_output_publication.as_ref().and_then(
                |admission| match &admission.publication {
                    RequiredOutputPublication::CoupledAuthored { group } => Some(group),
                    _ => None,
                },
            )?;
        let part = group.stages.get(stage)?;
        (part.source == source).then_some(RelationalCoupledContribution {
            records: &part.records,
            publication_required: false,
            ordinary: Some((&part.roles, &part.output_ids)),
        })
    }
    /// Actual per-writer changes from the applied assembly, before publication.
    pub fn coupled_contribution_records(
        &self,
        producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Option<RelationalCoupledContribution<'_>> {
        self.state
            .required_output_publication
            .iter()
            .find_map(|admission| {
                let RequiredOutputPublication::CoupledAuthored { group } = &admission.publication
                else {
                    return None;
                };
                group.parts.iter().find_map(|part| {
                    let (actual, _) = part.publication.lineage_attribution()?;
                    let actual_meaning = part.publication.producing_meaning()?;
                    (actual == producer && actual_meaning == meaning).then_some(
                        RelationalCoupledContribution {
                            records: &part.records,
                            publication_required: part.publication.owned().is_some(),
                            ordinary: match (&part.publication, &part.ordinary_ids) {
                                (
                                    RelationalCoupledProducerPosture::Ordinary { roles, .. },
                                    Some(ids),
                                ) => Some((roles.as_slice(), ids)),
                                _ => None,
                            },
                        },
                    )
                })
            })
    }
    pub fn coupled_participants(
        &self,
    ) -> impl Iterator<Item = (&str, &RequiredOutputPublicationMeaning)> {
        self.state
            .required_output_publication
            .iter()
            .filter_map(|admission| match &admission.publication {
                RequiredOutputPublication::CoupledAuthored { group } => Some(group),
                _ => None,
            })
            .flat_map(|group| group.parts.iter())
            .map(|part| {
                let (producer, _) = part
                    .publication
                    .lineage_attribution()
                    .expect("Native admitted a producer participant");
                let meaning = part
                    .publication
                    .producing_meaning()
                    .expect("Native admitted producing origin");
                (producer, meaning)
            })
    }
}
