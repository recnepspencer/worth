//! Source-address publication dependencies, including the headless first output.

use std::sync::Arc;

mod observation;
mod publication_fence;
mod validation_observation;
pub(in crate::domain_computation::primary_graph) use publication_fence::SourcePublicationReadFence;

use im::OrdSet;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::PositionedRelationalSnapshot,
};

use super::super::{
    CoveredSourcePublication, RecordedSettlementIdentity, SemanticSource, SourcePublication,
};
use super::{
    admission::{IndexAdmission, RetainedIndexAdmission},
    derived::SettlementRegistrationStop,
    index_capacity,
    mark_state::{FullVerificationReason, MarkState},
    retention, InvalidationEditAdmission, SourceInvalidationOwner,
};
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
    _capacity: super::RetainedConsumedOutputCapacity,
    _backing_capacity: Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>,
}

mod registration;

/// Called by the actual performed/stable settlement registration, on its
/// prepared companion state. Only watchers of this native source partition
/// are visited; source-epoch or geometry equality never suppresses completion.
pub(super) fn record_completion(
    state: &mut MarkState,
    identity: &Arc<RecordedSettlementIdentity>,
    partition: [u8; 32],
    admission: &mut InvalidationEditAdmission,
) -> Result<OrdSet<Arc<RecordedSettlementIdentity>>, CompanionPreflightStop> {
    let key = PublicationSourceKey {
        source: identity.source().clone(),
        partition,
    };
    admission.ordered_read(state.source_publication_postings.len())?;
    let Some(targets) = state.source_publication_postings.get(&key).cloned() else {
        return Ok(OrdSet::new());
    };
    let mut all_affected = OrdSet::new();
    for address in targets {
        admission.work(1)?;
        admission.ordered_read(state.source_publications.len())?;
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
        let bytes = count
            .checked_mul(std::mem::size_of::<CompletedPublication>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(overflow())?;
        admission.index_bytes(
            bytes
                .checked_add(
                    index_capacity::arc_bytes::<SourcePublicationState>().ok_or(overflow())?,
                )
                .ok_or(overflow())?,
        )?;
        admission.index_bytes(
            index_capacity::arc_bytes::<RecordedSettlementIdentity>()
                .ok_or(overflow())?
                .checked_mul(count as u64)
                .ok_or(overflow())?,
        )?;
        admission.admit_read_scratch(bytes)?;
        admission.work(prior.heads.len() as u64)?;
        let mut heads = Vec::with_capacity(count);
        heads.extend(prior.heads.iter().filter(|head| head.key != key).cloned());
        heads.push(CompletedPublication {
            key: key.clone(),
            identity: Arc::clone(identity),
        });
        // Deterministic native dependency identity includes source AND partition.
        // Selection visits every initialized key; bound sort comparisons by n squared.
        let comparison_bytes = heads
            .iter()
            .try_fold(0u64, |sum, head| {
                sum.checked_add(head.key.source.output_binding.as_str().len() as u64)
            })
            .and_then(|sum| sum.checked_add((heads.len() as u64).checked_mul(8 + 4 * 32)?))
            .ok_or(overflow())?;
        admission.work(
            comparison_bytes
                .checked_mul(heads.len() as u64)
                .ok_or(overflow())?,
        )?;
        heads.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        state.source_publication_head_count = state
            .source_publication_head_count
            .checked_sub(prior.heads.len())
            .and_then(|n| n.checked_add(count))
            .ok_or(overflow())?;
        admission.index_edit::<Arc<OutputSourceAddress>, Arc<SourcePublicationState>>(
            state.source_publications.len(),
        )?;
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
            admission,
            &mut counts,
        )?;
        let affected = super::delivery::propagate(state, affected, admission, &mut counts)?;
        for identity in affected {
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(all_affected.len())?;
            all_affected.insert(identity);
        }
    }
    Ok(all_affected)
}

pub(super) fn retained_bytes(state: &MarkState) -> Option<u64> {
    index_capacity::retained_map_bytes::<Arc<OutputSourceAddress>, Arc<SourcePublicationState>>(
        state.source_publications.len(),
    )?
    .checked_add(
        index_capacity::arc_bytes::<OutputSourceAddress>()?
            .checked_mul(state.source_publications.len() as u64)?,
    )?
    .checked_add(
        index_capacity::arc_bytes::<SourcePublicationState>()?
            .checked_mul(state.source_publications.len() as u64)?,
    )?
    .checked_add(
        (std::mem::size_of::<CompletedPublication>() as u64)
            .checked_mul(state.source_publication_head_count as u64)?,
    )?
    .checked_add(
        index_capacity::arc_bytes::<RecordedSettlementIdentity>()?
            .checked_mul(state.source_publication_head_count as u64)?,
    )?
    .checked_add(
        (2 * std::mem::size_of::<usize>() as u64)
            .checked_mul(state.source_publications.len() as u64)?,
    )?
    .checked_add(index_capacity::retained_map_bytes::<
        PublicationSourceKey,
        OrdSet<Arc<OutputSourceAddress>>,
    >(state.source_publication_postings.len())?)?
    .checked_add(index_capacity::retained_forest_bytes::<
        Arc<OutputSourceAddress>,
        (),
    >(
        state.source_publication_posting_count,
        state.source_publication_postings.len(),
    )?)
}

fn overflow() -> CompanionPreflightStop {
    CompanionPreflightStop::PreparationMemoryCounterOverflow
}
