//! The canonical inverse has an exact fresh-generation anchor and a bijective roster.
use super::*;
use crate::{
    history::data::CodecRetirementClosureStop as Stop,
    publication::patch::data::RecordStructuralChange as Change, transactions::data::RecordRef,
};
use worth_execution::MapKernelContext;

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn validate_restoration_header(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let Some(event) = &self.authored_cascade_restoration else {
            return Ok(());
        };
        let charge = super::super::super::codec_retirement_closure::charge;
        charge(
            context,
            self.patch.authoritative_record_patches.len() + event.entities.len(),
        )?;
        if self.producer_cascade_retirement.is_some()
            || self.required_output_origin.is_some()
            || self.authored_cascade_retirement.is_some()
            || event.publications.is_empty()
            || event.original_anchor == event.anchor
            || !event
                .entities
                .contains(&(event.original_anchor, event.anchor))
            || !self.patch.authoritative_record_patches.iter().any(|patch| {
                patch.target == RecordRef::Entity(event.anchor)
                    && patch.structural_change == Change::Created
            })
        {
            return Err("invalid recorded authored restoration anchor".into());
        }
        for (index, (old, new)) in event.entities.iter().enumerate() {
            charge(context, index + 1)?;
            if old == new
                || event.entities[..index]
                    .iter()
                    .any(|(a, b)| a == old || b == new)
            {
                return Err("recorded restoration repeats or revives a native generation".into());
            }
        }
        for (index, (old, new)) in event.relations.iter().enumerate() {
            charge(
                context,
                index + self.patch.authoritative_record_patches.len() + 1,
            )?;
            if old == new
                || event.relations[..index]
                    .iter()
                    .any(|(a, b)| a == old || b == new)
                || !self.patch.authoritative_record_patches.iter().any(|patch| {
                    patch.target == RecordRef::Relation(*new)
                        && patch.structural_change == Change::Created
                })
            {
                return Err("restoration repeats or invents a relation allocation".into());
            }
        }
        let mut edges = 0usize;
        let mut members = 0usize;
        for (index, origin) in event.publications.iter().enumerate() {
            charge(
                context,
                origin.members.len() + event.entities.len().saturating_mul(origin.members.len()),
            )?;
            self.validate_output_origin_header(origin)?;
            if !origin.source_scope_owned
                || !matches!(origin.publication, CanonicalOutputPublication::AuthoredRestoration { retirement, .. } if retirement == event.retirement)
                || origin.members.iter().any(|member| {
                    member.id == event.anchor
                        || member.posture != crate::mvcc::RequiredOutputRolePosture::Create
                        || !event.entities.iter().any(|(_, id)| *id == member.id)
                })
            {
                return Err("restoration omits exact recorded members".into());
            }
            charge(
                context,
                origin
                    .relations
                    .len()
                    .saturating_mul(event.relations.len() + 1),
            )?;
            edges = edges
                .checked_add(origin.relations.len())
                .ok_or("restoration relation count overflows")?;
            if origin
                .relations
                .iter()
                .any(|edge| !event.relations.iter().any(|(_, id)| *id == edge.id))
            {
                return Err("restoration relation has no recorded inverse".into());
            }
            members = members
                .checked_add(origin.members.len())
                .ok_or("restoration member count overflows")?;
            for prior in &event.publications[..index] {
                charge(
                    context,
                    prior.members.len().saturating_mul(origin.members.len())
                        + prior.meaning.family.len()
                        + origin.meaning.family.len()
                        + 64,
                )?;
                if prior.meaning.same_route(&origin.meaning)
                    || origin
                        .outputs
                        .iter()
                        .any(|id| prior.outputs.binary_search(id).is_ok())
                {
                    return Err("restoration repeats an original route or member".into());
                }
            }
            let mut source = origin.meaning.source_scope;
            for _ in 0..event.publications.len() {
                if source == event.anchor {
                    break;
                }
                let mut parent = None;
                for group in &event.publications {
                    charge(context, group.members.len() + 1)?;
                    if group.outputs.binary_search(&source).is_ok() {
                        parent = Some(group.meaning.source_scope);
                    }
                }
                source = parent.ok_or("restored publication has a foreign source")?;
            }
            if source != event.anchor {
                return Err("restored source ownership is cyclic".into());
            }
        }
        if edges != event.relations.len() {
            return Err("restoration adds an unowned relation".into());
        }
        if members.checked_add(1) != Some(event.entities.len()) {
            return Err("restoration adds an unowned entity".into());
        }
        Ok(())
    }
}
