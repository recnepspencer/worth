//! Current publication obligations projected through the one declared ownership reader.
use contracts::branch::model_ownership::read_denial::RelationalModelOwnershipDenial;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::model_ownership::basis;
use worth_relational_branch::internal_port::branch::model_ownership::traversal::Read;
use worth_relational_branch::internal_port::publication::ownership::read::roster::CurrentPublicationRoster;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::branch::model_ownership::selection::RelationalModelOwnershipOwnerPort as _;
use worth_relational_mutation::internal_port::branch::model_ownership::RelationalModelOwnership;

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
        publication: contracts::history::data::coordinates::CommitId,
        meaning: &'a contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    },
}
/// Borrowed current-fact evidence, constructible only by this custody projection.
#[derive(Clone, Copy)]
pub struct RelationalOwnedModelPublishedRoute<'a> {
    runtime: u64,
    origin: &'a worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin,
}
impl RelationalOwnedModelPublishedRoute<'_> {
    pub fn meaning(&self) -> &contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning{
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
                contracts::execution::request_stage::checkpoint(
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

pub fn observe(
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
            .ok_or(Denial::Native(contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
        let _records_memory = execution.reserve_retained_memory(bytes).map_err(Denial::Request)?;
        contracts::execution::request_stage::checkpoint(context, entities.len() as u64)
            .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        let records: Vec<_> = entities.iter().copied().map(RecordRef::Entity).collect();
        let roster = crate::publication::ownership::read(execution, selected, &records, context)
            .map_err(Denial::Custody)?;
        for origin in roster.origins().chain(roster.codec_retired_origins()) {
            contracts::execution::request_stage::checkpoint(context, entities.len().checked_ilog2().unwrap_or(0) as u64 + 1)
                .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
            if !entities.contains(&origin.meaning.source_scope) {
                return Err(Denial::Custody(contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                    contracts::publication::data::publication_error::PublicationError::authored_retirement(
                        contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                            publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                        }, RecordRef::Entity(origin.meaning.source_scope)) )));
            }
        }
        Ok(RelationalOwnedModelPublicationRead { basis: selected.clone(), roster })
    }).map_err(Denial::Request)?.0
}
