//! Current owned publications whose exact source would be removed by a candidate.
use super::{charge, CodecRetirementClosureStop as Stop, PublicationHistoryFailure as Failure};
use crate::{
    history::data::CanonicalRequiredOutputOrigin, identity::data::EntityId,
    mvcc::RequiredOutputRolePosture,
};
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
pub(crate) fn select_owned_scope_dependent(
    mut selection: OwnedScopeDependentSelection<'_>,
    current: &crate::publication::ownership::CurrentPublicationRoster,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, Failure> {
    for origin in current.origins() {
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

/// Surviving opted-in writers whose owned edges consume a retiring authored source
/// or a member of its admitted source-scope closure. Selection uses the closure
/// owner's already-authenticated current publications; this never walks history.
pub(super) fn linked_owners(
    anchors: &[EntityId],
    retiring: &[crate::history::data::SelectedOutputPublication],
    current: &[crate::history::data::SelectedOutputPublication],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<crate::history::data::SelectedOutputPublication>, Stop> {
    let mut selected = Vec::new();
    for publication in current {
        let origin = &publication.origin;
        charge(context, retiring.len() + origin.members.len() + 1)?;
        if !origin.source_scope_owned
            || origin
                .members
                .iter()
                .all(|m| m.posture == RequiredOutputRolePosture::Retire)
            || retiring.iter().any(|p| p.origin.same_publication(origin))
        {
            continue;
        }
        let mut linked = false;
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
        }
    }
    Ok(selected)
}
