//! Index this candidate's actual writes once; ownership remains the shared selector's.
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use super::*;
use branch::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::record_ids::EntityId;
use contracts::identity::data::record_ids::RelationId;
use std::collections::{HashMap, HashSet};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
type Route<'a> = (
    &'a str,
    EntityId,
    worth_foundational::CanonicalDigestId,
    CommitId,
);
pub(super) struct CandidateWrites<'a> {
    destroyed: HashMap<RecordRef, usize>,
    updated: HashMap<RelationId, usize>,
    after: HashMap<RelationId, (EntityId, EntityId)>,
    successors: HashSet<Route<'a>>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<'a> CandidateWrites<'a> {
    pub(super) fn new(
        execution: &ExecutionRequest<'_, '_>,
        candidate: &'a CanonicalCommitEnvelopeBody,
        adjacency: &[contracts::authority::mutation::adjacency_delta::AdjacencyDelta],
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        let patch = &candidate.patch.authoritative_record_patches;
        let rows = patch
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(adjacency.len()))
            .and_then(|n| n.checked_add(candidate.output_origins().count()))
            .and_then(|n| {
                n.checked_add((*candidate.owner_required_output_codec_retirements()).len())
            })
            .and_then(|n| n.checked_add(candidate.historical_absent_routes().count()))
            .and_then(|n| {
                n.checked_mul(4 * std::mem::size_of::<(RecordRef, (EntityId, EntityId))>())
            })
            .ok_or_else(|| {
                request(contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
            })?;
        let funding = execution
            .reserve_retained_memory(rows as u64)
            .map_err(request)?;
        let mut selected = Self {
            destroyed: HashMap::new(),
            updated: HashMap::new(),
            after: HashMap::new(),
            successors: HashSet::new(),
            _funding: funding,
        };
        for (ordinal, patch) in patch.iter().enumerate() {
            contracts::publication::ownership::read_request::checkpoint(context, 1)?;
            if matches!(
                patch.structural_change,
                Change::Deleted | Change::RetainedForAudit | Change::MaterializationSuspended
            ) {
                selected
                    .destroyed
                    .entry(patch.target.clone())
                    .or_insert(ordinal);
            } else if let RecordRef::Relation(id) = patch.target
                && patch.structural_change == Change::Updated
            {
                selected.updated.entry(id).or_insert(ordinal);
            }
        }
        for delta in adjacency {
            contracts::publication::ownership::read_request::checkpoint(context, 1)?;
            if let contracts::authority::mutation::adjacency_delta::AdjacencyDeltaKind::Created {
                source,
                target,
            } = delta.kind
            {
                selected.after.insert(delta.relation_id, (source, target));
            }
        }
        for successor in candidate.output_origins() {
            contracts::publication::ownership::read_request::checkpoint(
                context,
                successor.meaning.family.len() as u64 + 1,
            )?;
            if let Some(prior) = successor.publication.predecessor() {
                selected.successors.insert((
                    &successor.meaning.family,
                    successor.meaning.source_scope,
                    successor.meaning.source_partition,
                    prior,
                ));
            }
        }
        for absence in candidate.historical_absent_routes() {
            contracts::publication::ownership::read_request::checkpoint(
                context,
                absence.meaning.family.len() as u64 + 1,
            )?;
            selected.successors.insert((
                &absence.meaning.family,
                absence.meaning.source_scope,
                absence.meaning.source_partition,
                absence.publication,
            ));
        }
        for retirement in candidate.owner_required_output_codec_retirements() {
            contracts::publication::ownership::read_request::checkpoint(
                context,
                retirement.meaning.family.len() as u64 + 1,
            )?;
            selected.successors.insert((
                &retirement.meaning.family,
                retirement.meaning.source_scope,
                retirement.meaning.source_partition,
                retirement.publication,
            ));
        }
        Ok(selected)
    }
    pub(super) fn covered(&self, origin: &CanonicalRequiredOutputOrigin) -> bool {
        self.successors.contains(&(
            origin.meaning.family.as_str(),
            origin.meaning.source_scope,
            origin.meaning.source_partition,
            origin.commit.commit_id,
        ))
    }
    pub(super) fn destruction(
        &self,
        origin: &CanonicalRequiredOutputOrigin,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<RecordRef>, Stop> {
        let mut first = None;
        let mut consider = |record: RecordRef| {
            if let Some(&ordinal) = self.destroyed.get(&record) {
                choose(&mut first, ordinal, record);
            }
        };
        visit(context, origin.meaning.family.len() + 1)?;
        consider(RecordRef::Entity(origin.meaning.source_scope));
        for member in &origin.members {
            visit(context, 1)?;
            if member.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire {
                consider(RecordRef::Entity(member.id));
            }
        }
        for edge in &origin.relations {
            visit(context, 1)?;
            consider(RecordRef::Relation(edge.id));
            consider(RecordRef::Entity(edge.source));
            consider(RecordRef::Entity(edge.target));
        }
        for edge in &origin.relations {
            visit(context, 1)?;
            if let Some(&ordinal) = self.updated.get(&edge.id)
                && self
                    .after
                    .get(&edge.id)
                    .is_some_and(|&(source, target)| source != edge.source || target != edge.target)
            {
                choose(&mut first, ordinal, RecordRef::Relation(edge.id));
            }
        }
        Ok(first.map(|(_, record)| record))
    }
}
fn choose(first: &mut Option<(usize, RecordRef)>, ordinal: usize, record: RecordRef) {
    if first.as_ref().is_none_or(|(prior, _)| ordinal < *prior) {
        *first = Some((ordinal, record));
    }
}
fn visit(context: &mut worth_execution::MapKernelContext<'_, '_>, work: usize) -> Result<(), Stop> {
    contracts::execution::request_stage::checkpoint(context, work as u64)
        .map_err(|cause| Stop::Request(cause.into()))
}
