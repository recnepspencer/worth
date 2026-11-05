//! A selection header cannot impersonate a birth, retirement or ordinary producer run.
use super::*;
use crate::history::data::CodecRetirementClosureStop as Stop;
impl CanonicalCommitEnvelope {
    pub(in crate::history::data::canonical_commit_envelope::required_output_origin) fn validate_historical_selection_header(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let Some(event) = &self.historical_output_selection else {
            return Ok(());
        };
        if !matches!(&self.authority_kind,
                super::super::super::CanonicalCommitAuthorityKind::OwnedGraphSelection(graph)
                    if graph.selected_basis() == &event.selected_basis)
            || event.publications.is_empty()
            || self.required_output_origin.is_some()
            || self.producer_cascade_retirement.is_some()
            || self.authored_cascade_retirement.is_some()
            || self.authored_cascade_restoration.is_some()
            || self.coupled_authored_publication.is_some()
            || !self.is_unmerged_output_publication_step()
        {
            return Err("mixed or empty historical publication selection".into());
        }
        for (index, origin) in event.publications.iter().enumerate() {
            context
                .checkpoint(index as u64 + 1)
                .map_err(|cause| Stop::Request(cause.into()))?;
            self.validate_output_origin_header(origin)?;
            if !origin.source_scope_owned
                || !matches!(origin.publication,
                CanonicalOutputPublication::HistoricalSelection { selected, generation, .. }
                    if selected != self.commit.commit_id && generation.0 != 0)
                || event.publications[..index]
                    .iter()
                    .any(|p| p.meaning.same_route(&origin.meaning))
            {
                return Err("invalid historical producer selection route".into());
            }
        }
        Ok(())
    }
}
