//! An opted-in publication is scoped by a live entity of its installed source kind.
use super::*;
use crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::MapKernelContext;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    basis: &AdmittedRelationalBranchBasis,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    producer: &str,
    meaning: &RequiredOutputPublicationMeaning,
    previous: Option<&CanonicalRequiredOutputOrigin>,
    publication: contracts::history::data::coordinates::CommitId,
    state: &WorkingState,
    retiring: bool,
    source_input: Option<
        &crate::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedRead<
            crate::visibility::materialization::read_records::reader::pinned_read::RelationalAppliedCandidateReadIdentity,
        >,
    >,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    contracts::execution::request_stage::checkpoint(
        context,
        installed.source_scope_ownership.lookup_work(producer) as u64,
    )
    .map_err(origin_request)?;
    let kind = installed.source_scope_ownership.scope_kind(producer);
    let owned = kind.is_some();
    let invalid = || {
        TransactionCommitError::publication(
            contracts::publication::data::publication_error::PublicationError::authored_retirement(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::CorruptPublication,
                RecordRef::Entity(meaning.source_scope),
            ),
        )
    };
    if previous.is_some_and(|prior| prior.source_scope_owned != owned) {
        return Err(invalid());
    }
    let Some(kind) = kind else { return Ok(()) };
    installed.require_current_ownership_storage(basis, meaning.source_scope)?;
    if source_input.is_some_and(|input| {
        input.candidate_identity().proposal().branch_observation() != basis.reference()
    }) {
        return Err(invalid());
    }
    let source_root: &dyn worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess =
        match source_input {
            // The pin's read residency lends the candidate's source regions.
            Some(input) => input.resident_basis().basis(),
            None => basis,
        };
    let row = source_root
        .get_current_partition(meaning.source_scope.partition_id)
        .and_then(|partition| {
            partition
                .entity_arena()
                .get(&meaning.source_scope)
                .map(|row| (row.is_live(), row.kind_id()))
        });
    let after = OverlayStateView::new(basis, state);
    let live_after = after
        .get_current_partition(meaning.source_scope.partition_id)
        .and_then(|partition| {
            partition
                .entity_arena()
                .get(&meaning.source_scope)
                .map(|row| row.is_live() && row.kind_id() == Some(kind))
        })
        .unwrap_or(false);
    if row != Some((true, Some(kind))) || (!retiring && !live_after) {
        let publication = previous.map_or(publication, |prior| prior.commit.commit_id);
        return Err(TransactionCommitError::publication(
            contracts::publication::data::publication_error::PublicationError::owned_publication(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::PublicationOwnerUnavailable {
                    publication,
                    owner: meaning.source_scope,
                },
                previous
                    .and_then(|prior| prior.outputs.first())
                    .copied()
                    .unwrap_or(meaning.source_scope),
                meaning.source_scope,
            ),
        ));
    }
    Ok(())
}

/// The per-publication installer accepts owned retirement only inside Native's
/// private, already admitted cascade installer, never from a caller's warm capsule.
pub(super) fn require_cascade(
    origin: &CanonicalRequiredOutputOrigin,
    cascade: Option<&worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::authored_cascade::CanonicalAuthoredRetirement>,
) -> Result<(), TransactionCommitError> {
    if origin.source_scope_owned && cascade.is_none() {
        return Err(TransactionCommitError::publication(
            contracts::publication::data::publication_error::PublicationError::owned_publication(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                    publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                }, origin.outputs[0], origin.meaning.source_scope,
            ),
        ));
    }
    Ok(())
}
