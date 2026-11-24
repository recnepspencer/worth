//! Current owned publications whose exact source would be removed by a candidate.
use super::{charge, CodecRetirementClosureStop as Stop, PublicationHistoryFailure as Failure};
use crate::history::data::CanonicalRequiredOutputOrigin;
use crate::identity::data::EntityId;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture;
use std::sync::Arc;
use worth_execution::MapKernelContext;

pub(crate) enum OwnedScopeDependentSelection<'a> {
    RemovedSources(&'a [EntityId]),
    LiveRecord(
        &'a mut dyn FnMut(
            &CanonicalRequiredOutputOrigin,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<bool, Stop>,
    ),
}

/// One current-fact selection owner serves source removal and the write fence.
pub(crate) fn select_owned_scope_dependent<'a>(
    mut selection: OwnedScopeDependentSelection<'_>,
    origins: impl Iterator<Item = &'a Arc<CanonicalRequiredOutputOrigin>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, Failure> {
    for origin in origins {
        crate::execution::request_stage::checkpoint(context, 1).map_err(|cause| {
            Failure::at_source(origin.meaning.source_scope, Stop::Request(cause.into()))
        })?;
        let matches = match &mut selection {
            OwnedScopeDependentSelection::RemovedSources(removed) => {
                crate::execution::request_stage::checkpoint(context, removed.len() as u64)
                    .map_err(|cause| {
                        Failure::at_source(origin.meaning.source_scope, Stop::Request(cause.into()))
                    })?;
                removed.contains(&origin.meaning.source_scope)
            }
            OwnedScopeDependentSelection::LiveRecord(matches) => matches(origin, context)
                .map_err(|cause| Failure::at_source(origin.meaning.source_scope, cause))?,
        };
        if matches {
            return Ok(Some(Arc::clone(origin)));
        }
    }
    Ok(None)
}

/// Surviving opted-in writers reached by actual retained required inputs or owned
/// edges of a retiring authored source and its admitted source-scope closure. Selection uses the closure
/// owner's already-authenticated current publications; this never walks history.
pub(crate) fn linked_owners(
    anchors: &[EntityId],
    retiring: &[crate::history::data::SelectedOutputPublication],
    current: &[crate::history::data::SelectedOutputPublication],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<crate::history::data::SelectedOutputPublication>, Stop> {
    let mut selected: Vec<crate::history::data::SelectedOutputPublication> = Vec::new();
    loop {
        let mut added = false;
        for publication in current {
            let origin = &publication.origin;
            charge(
                context,
                retiring.len() + origin.members.len() + selected.len() + 1,
            )?;
            if !origin.source_scope_owned
                || origin
                    .members
                    .iter()
                    .all(|m| m.posture == RequiredOutputRolePosture::Retire)
                || retiring.iter().any(|p| p.origin.same_publication(origin))
                || selected.iter().any(|p| p.origin.same_publication(origin))
            {
                continue;
            }
            let mut linked = false;
            for input in &origin.inputs {
                charge(context, anchors.len() + 1)?;
                linked |= anchors.contains(&input.source_scope);
                for parent in retiring.iter().chain(&selected) {
                    charge(
                        context,
                        input.family.len() + parent.origin.meaning.family.len() + 65,
                    )?;
                    linked |= input.same_route(&parent.origin.meaning);
                }
            }
            for edge in &origin.relations {
                charge(context, anchors.len() + 1)?;
                linked |= anchors.contains(&edge.source) || anchors.contains(&edge.target);
                for removed in retiring {
                    charge(context, removed.origin.members.len())?;
                    linked |= removed.origin.members.iter().any(|m| {
                        m.posture != RequiredOutputRolePosture::Retire
                            && (m.id == edge.source || m.id == edge.target)
                    });
                }
            }
            if linked {
                selected.push(publication.clone());
                added = true;
            }
        }
        if !added {
            return Ok(selected);
        }
    }
}
