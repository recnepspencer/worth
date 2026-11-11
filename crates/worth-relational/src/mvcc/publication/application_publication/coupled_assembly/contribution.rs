//! Recorded applied participants, projected from the one Native group owner.
use super::*;
impl crate::mvcc::ValidatedRelationalProposal {
    /// Actual per-writer changes from the applied assembly, before publication.
    pub fn coupled_contribution_records(
        &self,
        producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Option<&[crate::transactions::data::RecordRef]> {
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
                    let RequiredOutputPublication::Preserve {
                        meaning: actual_meaning,
                        ..
                    } = &part.publication
                    else {
                        return None;
                    };
                    (actual == producer && actual_meaning == meaning)
                        .then_some(part.records.as_slice())
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
                let RequiredOutputPublication::Preserve { meaning, .. } = &part.publication else {
                    unreachable!("Native admitted Preserve")
                };
                (producer, meaning)
            })
    }
}
