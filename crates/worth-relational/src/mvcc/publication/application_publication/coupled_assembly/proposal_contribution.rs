//! Inspect the exact applied participants of a live validated proposal.
use super::*;

impl crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal {
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
        contribution::authored_stage(group.stages.get(stage)?, source)
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
                contribution::produced(&group.parts, producer, meaning)
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
            .map(contribution::route)
    }
}
