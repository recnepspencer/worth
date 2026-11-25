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
                CanonicalCoupledAuthoredSources::Edit { .. } => {}
                CanonicalCoupledAuthoredSources::Retirement {
                    prefix: source,
                    later,
                }
                | CanonicalCoupledAuthoredSources::MemberEditRetirement {
                    prefix: source,
                    sources: later,
                } => {
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
                        retiring.iter().map(|item| item.origin.as_ref()),
                        std::iter::empty(),
                        source.publications.iter().map(Arc::as_ref),
                        false,
                        context,
                    )?;
                    let mut required = super::super::scope_dependents::linked_owners(
                        &anchors, &retiring, &preceding, context,
                    )?;
                    let mut optional = Vec::new();
                    if !group.sources.is_edit() {
                        include_later(
                            &required,
                            &mut optional,
                            later,
                            &preceding,
                            retiring.iter().map(|item| item.origin.as_ref()),
                            context,
                        )?;
                    }
                    exact(
                        required.iter().map(|item| item.origin.as_ref()),
                        optional.iter().map(|item| item.origin.as_ref()),
                        group
                            .participants
                            .iter()
                            .filter(|part| part.is_required())
                            .map(|part| part.origin.as_ref()),
                        true,
                        context,
                    )?;
                }
                CanonicalCoupledAuthoredSources::Restoration {
                    prefix: source,
                    later,
                }
                | CanonicalCoupledAuthoredSources::MemberEditRestoration {
                    prefix: source,
                    sources: later,
                } => {
                    charge(context, ancestry.len() + 1)?;
                    let original_envelope = ancestry
                        .iter()
                        .find(|item| item.commit.commit_id == source.retirement)
                        .ok_or_else(|| {
                            Failure::at_source(
                                source.original_anchor,
                                "coupled restore lost retirement".into(),
                            )
                        })?;
                    let original = original_envelope.coupled_authored_publication.as_ref();
                    let retirement =
                        original_envelope
                            .authored_source_retirement()
                            .ok_or_else(|| {
                                Failure::at_source(
                                    source.original_anchor,
                                    "coupled restore lost its genuine retirement".into(),
                                )
                            })?;
                    let mut required = Vec::new();
                    for part in original
                        .iter()
                        .flat_map(|original| &original.participants)
                        .filter(|part| part.is_required())
                    {
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
                    let mut optional = Vec::new();
                    if !group.sources.is_edit() {
                        include_later(
                            &required,
                            &mut optional,
                            later,
                            &preceding,
                            retirement.publications.iter().map(Arc::as_ref),
                            context,
                        )?;
                    }
                    exact(
                        required.iter().map(|item| item.origin.as_ref()),
                        optional.iter().map(|item| item.origin.as_ref()),
                        group
                            .participants
                            .iter()
                            .filter(|part| part.is_required())
                            .map(|part| part.origin.as_ref()),
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
                evidence: crate::publication::ownership::evidence::PublicationEvidence::canonical(
                    Arc::clone(envelope),
                ),
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

/// Current owners of later edit stages' sources survive a retirement or
/// restoration prefix as they survive a lone edit: each may republish, none
/// must (public #348). Records that list one as a required owner still validate.
fn include_later<'a>(
    required: &[SelectedOutputPublication],
    optional: &mut Vec<SelectedOutputPublication>,
    later: &[EntityId],
    preceding: &[SelectedOutputPublication],
    retired: impl ExactSizeIterator<Item = &'a CanonicalRequiredOutputOrigin> + Clone,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Failure> {
    for source in later {
        for owner in later_source_owners(*source, preceding, context)? {
            for other_family_bytes in retired
                .clone()
                .map(|item| item.meaning.family.len())
                .chain(
                    required
                        .iter()
                        .chain(optional.iter())
                        .map(|item| item.origin.meaning.family.len()),
                )
            {
                charge(
                    context,
                    owner.origin.meaning.family.len() + other_family_bytes + 65,
                )?;
            }
            if !retired
                .clone()
                .any(|item| item.same_publication(&owner.origin))
                && !required
                    .iter()
                    .chain(optional.iter())
                    .any(|item| item.origin.same_publication(&owner.origin))
            {
                optional.push(owner);
            }
        }
    }
    Ok(())
}

/// Every `required` route is recorded, and every recorded route is required
/// or `optional`; each succeeds its exact predecessor.
fn exact<'a, 'b>(
    required: impl Iterator<Item = &'b CanonicalRequiredOutputOrigin> + Clone,
    optional: impl Iterator<Item = &'b CanonicalRequiredOutputOrigin> + Clone,
    recorded: impl Iterator<Item = &'a CanonicalRequiredOutputOrigin> + Clone,
    preserve: bool,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Failure> {
    let succeeds = |before: &CanonicalRequiredOutputOrigin, after: &CanonicalRequiredOutputOrigin| {
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
        Ok(())
    };
    for before in required.clone() {
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
        succeeds(before, after)?;
    }
    for observed in recorded {
        for expected in required.clone().chain(optional.clone()) {
            charge(
                context,
                observed.meaning.family.len() + expected.meaning.family.len() + 65,
            )?;
        }
        if required
            .clone()
            .any(|item| item.meaning.same_route(&observed.meaning))
        {
            continue;
        }
        let before = optional
            .clone()
            .find(|item| item.meaning.same_route(&observed.meaning))
            .ok_or_else(|| {
                Failure::at_source(
                    observed.meaning.source_scope,
                    "coupled group contains an extra owner".into(),
                )
            })?;
        succeeds(before, observed)?;
    }
    Ok(())
}

pub(super) mod tests;
