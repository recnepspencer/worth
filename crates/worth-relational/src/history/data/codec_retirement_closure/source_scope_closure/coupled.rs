//! Group completeness derives from preceding canonical facts, not a claimed roster.
use super::*;
use crate::history::data::{
    CanonicalCoupledAuthoredSources, CanonicalOutputPublication, CanonicalRequiredOutputOrigin,
};

pub(super) fn validate_ancestry(
    ancestry: &[Arc<CanonicalCommitEnvelope>],
    bound: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Failure> {
    if ancestry.is_empty() {
        return Ok(());
    }
    let mut preceding: Vec<SelectedOutputPublication> = Vec::new();
    for envelope in ancestry.iter().rev() {
        if let Some(group) = &envelope.coupled_authored_publication {
            match &group.sources {
                CanonicalCoupledAuthoredSources::Retirement(source) => {
                    let anchors = source.anchors().collect::<Vec<_>>();
                    for (anchor, birth) in std::iter::once((source.anchor, source.birth))
                        .chain(source.co_anchors.iter().copied())
                    {
                        charge(context, ancestry.len() + 1)?;
                        let envelope = ancestry
                            .iter()
                            .find(|item| item.commit.commit_id == birth)
                            .ok_or_else(|| {
                                Failure::at_source(
                                    anchor,
                                    "coupled authored source lost birth".into(),
                                )
                            })?;
                        super::super::authored_cascade::require_authored_birth(
                            envelope, anchor, context,
                        )
                        .map_err(|stop| Failure::at_source(anchor, stop))?;
                        for publication in &preceding {
                            charge(context, publication.origin.members.len())?;
                            if publication
                                .origin
                                .members
                                .iter()
                                .any(|m| m.id == anchor && m.posture != Posture::Retire)
                            {
                                return Err(Failure::at_source(
                                    anchor,
                                    "coupled anchor belongs to a live publication".into(),
                                ));
                            }
                        }
                    }
                    let retiring = select(&anchors, &preceding, context)?;
                    exact(
                        &retiring,
                        source.publications.iter().map(Arc::as_ref),
                        false,
                        context,
                    )?;
                    let required = super::super::scope_dependents::linked_owners(
                        &anchors, &retiring, &preceding, context,
                    )?;
                    exact(
                        &required,
                        group.participants.iter().map(|part| part.origin.as_ref()),
                        true,
                        context,
                    )?;
                }
                CanonicalCoupledAuthoredSources::Restoration(source) => {
                    charge(context, ancestry.len() + 1)?;
                    let original = ancestry
                        .iter()
                        .find(|item| item.commit.commit_id == source.retirement)
                        .and_then(|item| item.coupled_authored_publication.as_ref())
                        .ok_or_else(|| {
                            Failure::at_source(
                                source.original_anchor,
                                "coupled restore lost retirement".into(),
                            )
                        })?;
                    let mut required = Vec::with_capacity(original.participants.len());
                    for part in &original.participants {
                        let prior = &part.origin;
                        for item in &preceding {
                            charge(
                                context,
                                prior.meaning.family.len() + item.origin.meaning.family.len() + 65,
                            )?;
                        }
                        let current = preceding
                            .iter()
                            .find(|item| {
                                item.origin.meaning.same_route(&prior.meaning)
                                    && item
                                        .origin
                                        .members
                                        .iter()
                                        .any(|m| m.posture != Posture::Retire)
                            })
                            .ok_or_else(|| {
                                Failure::at_source(
                                    prior.meaning.source_scope,
                                    "coupled restore owner is unavailable".into(),
                                )
                            })?;
                        required.push(current.clone());
                    }
                    exact(
                        &required,
                        group.participants.iter().map(|part| part.origin.as_ref()),
                        true,
                        context,
                    )?;
                }
            }
        }
        for (ordinal, origin) in envelope.output_origins().enumerate() {
            for prior in &preceding {
                charge(
                    context,
                    prior.origin.meaning.family.len() + origin.meaning.family.len() + 65,
                )?;
            }
            let selected = SelectedOutputPublication {
                envelope: Arc::clone(envelope),
                origin: Arc::clone(origin),
                ordinal,
            };
            if let Some(prior) = preceding
                .iter_mut()
                .find(|item| item.origin.meaning.same_route(&origin.meaning))
            {
                *prior = selected;
            } else {
                if preceding.len() >= bound {
                    return Err(Failure::at_source(
                        origin.meaning.source_scope,
                        "coupled source catalog exceeds bound".into(),
                    ));
                }
                preceding.push(selected);
            }
        }
    }
    Ok(())
}

fn exact<'a>(
    required: &[SelectedOutputPublication],
    recorded: impl Iterator<Item = &'a CanonicalRequiredOutputOrigin> + Clone,
    preserve: bool,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Failure> {
    for expected in required {
        let before = &expected.origin;
        for observed in recorded.clone() {
            charge(
                context,
                before.meaning.family.len() + observed.meaning.family.len() + 65,
            )?;
        }
        let after = recorded
            .clone()
            .find(|item| item.meaning.same_route(&before.meaning))
            .ok_or_else(|| {
                Failure::at_source(
                    before.meaning.source_scope,
                    "coupled group omits required owner".into(),
                )
            })?;
        let valid = if preserve {
            matches!(after.publication, CanonicalOutputPublication::Preserve { predecessor } if predecessor == before.commit.commit_id)
        } else {
            matches!(after.publication, CanonicalOutputPublication::AuthoredRetirement { predecessor } if predecessor == before.commit.commit_id)
        };
        if !valid || !after.source_scope_owned {
            return Err(Failure::at_source(
                before.meaning.source_scope,
                "coupled group has a foreign predecessor".into(),
            ));
        }
    }
    for observed in recorded {
        for expected in required {
            charge(
                context,
                observed.meaning.family.len() + expected.origin.meaning.family.len() + 65,
            )?;
        }
        if !required
            .iter()
            .any(|item| item.origin.meaning.same_route(&observed.meaning))
        {
            return Err(Failure::at_source(
                observed.meaning.source_scope,
                "coupled group contains an extra owner".into(),
            ));
        }
    }
    Ok(())
}
