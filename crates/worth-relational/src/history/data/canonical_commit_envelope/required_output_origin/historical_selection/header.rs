//! A selection header cannot impersonate a birth, retirement or ordinary producer run.
use super::*;
use crate::history::data::CodecRetirementClosureStop as Stop;
impl CanonicalCommitEnvelope {
    pub(in crate::history::data) fn validate_historical_selection_header(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let Some(event) = &self.historical_output_selection else {
            return Ok(());
        };
        if !matches!(&self.authority_kind,
                super::super::super::CanonicalCommitAuthorityKind::OwnedGraphSelection(graph)
                    if graph.selected_basis() == &event.selected_basis)
            || (event.publications.is_empty() && event.absences.is_empty())
            || self.required_output_origin.is_some()
            || self.producer_cascade_retirement.is_some()
            || self.authored_cascade_retirement.is_some()
            || self.authored_cascade_restoration.is_some()
            || self.coupled_authored_publication.is_some()
            || !self.is_unmerged_output_publication_step()
        {
            return Err("mixed or empty historical publication selection".into());
        }
        for (index, absence) in event.absences.iter().enumerate() {
            context
                .checkpoint((absence.meaning.family.len() + 65) as u64)
                .map_err(|stop| Stop::Request(stop.into()))?;
            if absence.publication == self.commit.commit_id || absence.generation.0 == 0 {
                return Err("invalid historical publication absence".into());
            }
            for meaning in event
                .publications
                .iter()
                .map(|o| &o.meaning)
                .chain(event.absences[..index].iter().map(|a| &a.meaning))
            {
                context
                    .checkpoint((meaning.family.len() + absence.meaning.family.len() + 65) as u64)
                    .map_err(|stop| Stop::Request(stop.into()))?;
                if meaning.same_route(&absence.meaning) {
                    return Err("mixed historical publication route".into());
                }
            }
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
