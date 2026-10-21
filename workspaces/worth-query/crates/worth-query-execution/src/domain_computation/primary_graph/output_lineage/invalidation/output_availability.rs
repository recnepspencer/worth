//! Source-address publication dependencies, including the headless first output.

use std::sync::Arc;

mod observation;
mod publication_fence;
mod validation_observation;
pub(in crate::domain_computation::primary_graph) use publication_fence::SourcePublicationReadFence;

use im::OrdSet;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::super::CoveredSourcePublication;
use super::super::RecordedSettlementIdentity;
use super::super::SemanticSource;
use super::super::SourcePublication;
use super::derived::SettlementRegistrationStop;
use super::mark_state::FullVerificationReason;
use super::mark_state::MarkState;
use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::application_contribution::OutputSourceAddress;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct PublicationSourceKey {
    source: SemanticSource,
    partition: [u8; 32],
}

/// The native completion set of one qualified family source. The selected
/// source epoch is held by each observation, not used to replace this address.
#[derive(Clone, Debug)]
pub(super) struct SourcePublicationState {
    heads: Arc<[CompletedPublication]>,
}

#[derive(Clone, Debug)]
pub(super) struct CompletedPublication {
    key: PublicationSourceKey,
    identity: Arc<RecordedSettlementIdentity>,
}

/// A pin of the original native image and actual source slot. No data-only
/// comparison can substitute a slot from another image or native source.
pub(in crate::domain_computation) struct ObservedSourcePublication {
    address: Arc<OutputSourceAddress>,
    state: Arc<SourcePublicationState>,
    candidate: Option<Arc<crate::domain_computation::primary_graph::invariant_projection::ObservedOutputCandidate>>,
    source_epoch: crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity,
    read_closure:
        Arc<crate::domain_computation::primary_graph::application_query::SourceQueryReadClosure>,
    selected: PositionedRelationalSnapshot,
    owner: std::sync::Weak<dyn observation::SourcePublicationReader>,


}

mod registration;

/// Called by the actual performed/stable settlement registration, on its
/// prepared companion state. Only watchers of this native source partition
/// are visited; source-epoch or geometry equality never suppresses completion.
pub(super) fn record_completion(
    state: &mut MarkState,
    identity: &Arc<RecordedSettlementIdentity>,
    partition: [u8; 32],
) -> Result<OrdSet<Arc<RecordedSettlementIdentity>>, CompanionPreflightStop> {
    let key = PublicationSourceKey {
        source: identity.source().clone(),
        partition,
    };

    let Some(targets) = state.source_publication_postings.get(&key).cloned() else {
        return Ok(OrdSet::new());
    };
    let mut all_affected = OrdSet::new();
    for address in targets {
        let Some(prior) = state.source_publications.get(&address).cloned() else {
            continue;
        };
        if prior
            .heads
            .iter()
            .any(|head| head.key == key && head.identity.as_ref() == identity.as_ref())
        {
            continue;
        }
        let count = prior
            .heads
            .iter()
            .filter(|head| head.key != key)
            .count()
            .checked_add(1)
            .ok_or(overflow())?;

        let mut heads = Vec::with_capacity(count);
        heads.extend(prior.heads.iter().filter(|head| head.key != key).cloned());
        heads.push(CompletedPublication {
            key: key.clone(),
            identity: Arc::clone(identity),
        });
        // Deterministic native dependency identity includes source AND partition.
        // Selection visits every initialized key; bound sort comparisons by n squared.

        heads.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        state.source_publication_head_count = state
            .source_publication_head_count
            .checked_sub(prior.heads.len())
            .and_then(|n| n.checked_add(count))
            .ok_or(overflow())?;

        state.source_publications.insert(
            Arc::clone(&address),
            Arc::new(SourcePublicationState {
                heads: heads.into(),
            }),
        );
        let mut counts = super::logical_marking::WorthQueryNativeMarkingWork::default();
        let affected = super::delivery::mark_keys(
            state,
            &[super::fact_key::FactPostingKey::SourcePublication(address)],
            &mut counts,
        )?;
        let affected = super::delivery::propagate(state, affected, &mut counts)?;
        for identity in affected {
            all_affected.insert(identity);
        }
    }
    Ok(all_affected)
}

fn overflow() -> CompanionPreflightStop {
    CompanionPreflightStop::PreparationMemoryCounterOverflow
}
