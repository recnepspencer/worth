//! Structural header checks reuse the existing authored and producer components.
use super::*;
use crate::history::data::CodecRetirementClosureStop as Stop;
use coupled_authored::{CanonicalCoupledAuthoredPublication, CanonicalCoupledAuthoredSources};
use worth_execution::MapKernelContext;

impl CanonicalCommitEnvelopeBody {
    pub(super) fn validate_coupled_authored_header(
        &self,
        event: &CanonicalCoupledAuthoredPublication,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let charge = super::super::super::codec_retirement_closure::charge;
        charge(context, event.participants.len() + 1)?;
        // An envelope records its prefix and at least one later stage; fewer
        // is an Edit, so a shorter list is a corrupt header, refused first.
        if matches!(&event.sources, CanonicalCoupledAuthoredSources::Stages(sources) if sources.len() < 2)
        {
            return Err("coupled authored stage list is shorter than two".into());
        }
        if matches!(&event.sources,
            CanonicalCoupledAuthoredSources::RetirementWithLaterSources { later, .. }
            | CanonicalCoupledAuthoredSources::RestorationWithLaterSources { later, .. }
            if later.is_empty())
        {
            return Err("corrective coupled stage list has no later source".into());
        }
        if self.required_output_origin.is_some()
            || self.authored_cascade_retirement.is_some()
            || self.authored_cascade_restoration.is_some()
            || self.producer_cascade_retirement.is_some()
            || event.participants.is_empty()
        {
            return Err("mixed or empty coupled authored publication".into());
        }
        match &event.sources {
            CanonicalCoupledAuthoredSources::Edit(_) => {}
            CanonicalCoupledAuthoredSources::Stages(_) => {}
            CanonicalCoupledAuthoredSources::Retirement(source)
            | CanonicalCoupledAuthoredSources::RetirementWithLaterSources {
                prefix: source, ..
            } => {
                self.validate_authored_retirement_components(source, true, context)?;
            }
            CanonicalCoupledAuthoredSources::Restoration(source)
            | CanonicalCoupledAuthoredSources::RestorationWithLaterSources {
                prefix: source, ..
            } => {
                self.validate_authored_restoration_components(source, true, context)?;
            }
        }
        for part in &event.participants {
            self.validate_output_origin_header(&part.origin)?;
            let producing = matches!(
                part.origin.publication,
                CanonicalOutputPublication::Preserve { .. }
            ) || (matches!(
                event.sources,
                CanonicalCoupledAuthoredSources::Edit(_)
                    | CanonicalCoupledAuthoredSources::Stages(_)
            ) && matches!(
                part.origin.publication,
                CanonicalOutputPublication::Birth
                    | CanonicalOutputPublication::BirthAfterCodecRetirement { .. }
            ));
            if (!part.origin.source_scope_owned
                && !matches!(
                    event.sources,
                    CanonicalCoupledAuthoredSources::Edit(_)
                        | CanonicalCoupledAuthoredSources::Stages(_)
                ))
                || !producing
            {
                return Err("coupled participant is not an opted-in producing origin".into());
            }
            self.validate_producer_retirement_components(
                Some(&part.origin),
                part.retirement.as_deref(),
                context,
            )?;
        }
        for (index, origin) in event.output_origins().enumerate() {
            for prior in event.output_origins().take(index) {
                charge(
                    context,
                    origin.meaning.family.len()
                        + prior.meaning.family.len()
                        + 64
                        + origin.members.len().saturating_mul(prior.members.len()),
                )?;
                charge(
                    context,
                    origin.relations.len().saturating_mul(
                        1 + (usize::BITS - prior.relations.len().leading_zeros()) as usize,
                    ),
                )?;
                if origin.relations.iter().any(|edge| {
                    prior
                        .relations
                        .binary_search_by_key(&edge.id, |edge| edge.id)
                        .is_ok()
                }) {
                    return Err("coupled publication repeats an owned relation".into());
                }
                if prior.meaning.same_route(&origin.meaning)
                    || origin
                        .members
                        .iter()
                        .any(|member| prior.outputs.binary_search(&member.id).is_ok())
                {
                    return Err("coupled publication repeats a route or output member".into());
                }
            }
        }
        Ok(())
    }
}

mod court_door;
