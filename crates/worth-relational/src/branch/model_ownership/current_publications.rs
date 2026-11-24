//! Current publication obligations projected through the one declared ownership reader.
use super::{basis, traversal::Read, RelationalModelOwnership, RelationalModelOwnershipDenial};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::EntityId;
use crate::publication::ownership::CurrentPublicationRoster;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;

/// Exact current facts, with roster/presence pairing already checked by custody.
/// This read grants no role, mutation, publication or source permission.
pub struct RelationalOwnedModelPublicationRead {
    basis: AdmittedRelationalBranchBasis,
    roster: CurrentPublicationRoster,
}
#[derive(Clone, Copy)]
pub enum RelationalOwnedModelOutputRoute<'a> {
    Published(RelationalOwnedModelPublishedRoute<'a>),
    CodecRetired {
        publication: crate::history::data::CommitId,
        meaning: &'a crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    },
}
/// Borrowed current-fact evidence, constructible only by this custody projection.
#[derive(Clone, Copy)]
pub struct RelationalOwnedModelPublishedRoute<'a> {
    runtime: u64,
    origin: &'a crate::history::data::CanonicalRequiredOutputOrigin,
}
impl RelationalOwnedModelPublishedRoute<'_> {
    pub fn meaning(&self) -> &crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning{
        &self.origin.meaning
    }
    /// Match the complete canonical coordinate, without issuing publication custody.
    pub fn matches_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        publication: &crate::mvcc::publication::application_publication::custody::RelationalRequiredOutputPublication,
    ) -> Result<bool, worth_execution::WorkCeilingDenial> {
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(
                    context,
                    (self.origin.meaning.family.len() + publication.meaning().family.len() + 96)
                        as u64,
                )?;
                Ok(self.runtime == publication.runtime_instance_id
                    && self.origin.same_publication(&publication.origin))
            })
            .map(|answer| {
                answer
                    .0
                    .map_err(worth_execution::WorkCeilingDenial::Stopped)
            })?
    }
}
impl RelationalOwnedModelPublicationRead {
    pub fn basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }
    pub fn routes(&self) -> impl Iterator<Item = RelationalOwnedModelOutputRoute<'_>> {
        self.roster
            .origins()
            .map(|origin| {
                RelationalOwnedModelOutputRoute::Published(RelationalOwnedModelPublishedRoute {
                    runtime: self.basis.identity().runtime_instance_id(),
                    origin,
                })
            })
            .chain(self.roster.codec_retired_origins().map(|origin| {
                RelationalOwnedModelOutputRoute::CodecRetired {
                    publication: origin.commit.commit_id,
                    meaning: &origin.meaning,
                }
            }))
    }
}

pub(super) fn observe(
    execution: &ExecutionRequest<'_, '_>,
    runtime_instance_id: u64,
    selected: &AdmittedRelationalBranchBasis,
    ownership: &RelationalModelOwnership,
    model: EntityId,
) -> Result<RelationalOwnedModelPublicationRead, RelationalModelOwnershipDenial> {
    use RelationalModelOwnershipDenial as Denial;
    execution.run_native_with_checkpoint(|context| {
        let (entities, _traversal_memory) = basis::read(runtime_instance_id, execution, selected, context, |read| {
            let kind = read.kind(model)?.ok_or(Denial::EntityUnavailable(model))?;
            ownership.owned_entities(read, model, kind)
        })?;
        let bytes = entities.len().checked_mul(std::mem::size_of::<RecordRef>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Denial::Native(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
        let _records_memory = execution.reserve_retained_memory(bytes).map_err(Denial::Request)?;
        crate::execution::request_stage::checkpoint(context, entities.len() as u64)
            .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        let records: Vec<_> = entities.iter().copied().map(RecordRef::Entity).collect();
        let roster = crate::publication::ownership::read(execution, selected, &records, context)
            .map_err(Denial::Custody)?;
        for origin in roster.origins().chain(roster.codec_retired_origins()) {
            crate::execution::request_stage::checkpoint(context, entities.len().checked_ilog2().unwrap_or(0) as u64 + 1)
                .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
            if !entities.contains(&origin.meaning.source_scope) {
                return Err(Denial::Custody(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                    crate::publication::data::PublicationError::authored_retirement(
                        crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                            publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                        }, RecordRef::Entity(origin.meaning.source_scope)) )));
            }
        }
        Ok(RelationalOwnedModelPublicationRead { basis: selected.clone(), roster })
    }).map_err(Denial::Request)?.0
}
