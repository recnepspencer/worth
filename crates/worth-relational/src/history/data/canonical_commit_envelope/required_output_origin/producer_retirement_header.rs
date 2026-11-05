//! Cold grouped retirement retains distinct routes rooted in actual generator deletions.
use super::*;
use crate::{
    history::data::CodecRetirementClosureStop as Stop, mvcc::RequiredOutputRolePosture as Posture,
    publication::patch::data::RecordStructuralChange as Change, transactions::data::RecordRef,
};
use worth_execution::MapKernelContext;
impl CanonicalCommitEnvelopeBody {
    pub(super) fn validate_producer_retirement_header(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let Some(event) = &self.producer_cascade_retirement else {
            if self.required_output_origin.as_ref().is_some_and(|origin| {
                matches!(
                    origin.publication,
                    CanonicalOutputPublication::ProducerRetirement { .. }
                )
            }) {
                return Err("producer retirement lacks its canonical group".into());
            }
            return Ok(());
        };
        let charge = super::super::super::codec_retirement_closure::charge;
        let root = self
            .required_output_origin
            .as_ref()
            .ok_or("producer cascade lost generator publication")?;
        self.validate_output_origin_header(root)?;
        if self.authored_cascade_retirement.is_some()
            || self.authored_cascade_restoration.is_some()
            || event.anchors.is_empty()
            || event.publications.is_empty()
            || !matches!(
                root.publication,
                CanonicalOutputPublication::Preserve { .. }
            )
        {
            return Err("mixed or empty producer cascade".into());
        }
        let roots = root
            .members
            .iter()
            .filter(|m| m.posture == Posture::Retire)
            .map(|m| m.id)
            .collect::<Vec<_>>();
        charge(
            context,
            root.members.len() + event.anchors.len().saturating_mul(roots.len() + 1),
        )?;
        if roots != event.anchors {
            return Err("producer cascade anchors differ from deleted generator membership".into());
        }
        for (index, origin) in event.publications.iter().enumerate() {
            self.validate_output_origin_header(origin)?;
            if !origin.source_scope_owned
                || !matches!(
                    origin.publication,
                    CanonicalOutputPublication::ProducerRetirement { .. }
                )
                || !origin.relations.is_empty()
            {
                return Err("producer cascade contains a live or unowned route".into());
            }
            for member in &origin.members {
                charge(
                    context,
                    root.members.len() + self.patch.authoritative_record_patches.len(),
                )?;
                if root.outputs.binary_search(&member.id).is_ok()
                    || !self.patch.authoritative_record_patches.iter().any(|p| {
                        p.target == RecordRef::Entity(member.id)
                            && p.structural_change == Change::Deleted
                    })
                {
                    return Err("producer cascade invents or overlaps membership".into());
                }
            }
            for prior in &event.publications[..index] {
                charge(
                    context,
                    origin.members.len().saturating_mul(prior.members.len())
                        + origin.meaning.family.len()
                        + prior.meaning.family.len()
                        + 64,
                )?;
                if prior.meaning.same_route(&origin.meaning)
                    || origin
                        .members
                        .iter()
                        .any(|m| prior.outputs.binary_search(&m.id).is_ok())
                {
                    return Err("producer cascade repeats a route or member".into());
                }
            }
            let mut source = origin.meaning.source_scope;
            for _ in 0..=event.publications.len() {
                charge(context, event.anchors.len())?;
                if event.anchors.contains(&source) {
                    break;
                }
                let mut parent = None;
                for candidate in &event.publications {
                    charge(context, candidate.members.len() + 1)?;
                    if candidate.outputs.binary_search(&source).is_ok() {
                        parent = Some(candidate.meaning.source_scope);
                    }
                }
                source = parent.ok_or("producer cascade has a foreign source")?;
            }
            if !event.anchors.contains(&source) {
                return Err("producer cascade ownership is cyclic".into());
            }
        }
        Ok(())
    }
}
