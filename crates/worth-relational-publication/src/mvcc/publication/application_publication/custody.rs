//! Runtime custody is issued from sealed performed results or cold native readmission.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::group_selection::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyFieldsPort as _};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort as _;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::execution::request_stage::checkpoint;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use std::sync::Arc;
pub mod current;
pub mod description;
pub(super) use current::readmit_successor;
pub use description::RecordedRequiredOutputRole;
pub mod prepared;
pub use prepared::prepare;
pub use prepared::{RelationalPerformedOutputPublications, RelationalPreparedOutputPublications};

#[derive(Clone, Debug)]
pub struct RelationalRequiredOutputPublication {
    pub(crate) runtime_instance_id: u64,
    pub(crate) evidence: worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence,
    pub(crate) origin: Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>,
    pub(crate) authority: worth_proof::RequiredOutputPublicationAuthority,
    pub(crate) readmitted_at: Option<worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis>,
}

impl RelationalRequiredOutputPublication {
    pub fn producer(&self) -> &str {
        &self.origin.producer
    }
    pub fn meaning(&self) -> &contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning{
        &self.origin.meaning
    }
    /// Actual performed roles, including retired members, in canonical native order.
    pub fn members(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &str,
            &str,
            contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture,
            contracts::identity::data::record_ids::EntityId,
        ),
    >{
        self.origin.members.iter().map(|member| {
            (
                member.role.as_str(),
                member.entity_name.as_str(),
                member.posture,
                member.id,
            )
        })
    }
    /// A genuine Native correction origin, independent of a producer execution locator.
    pub fn is_authored_restoration(&self) -> bool {
        matches!(
            self.origin.publication,
            worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication::AuthoredRestoration { .. }
        )
    }
    /// This Native origin selects the producer's own retained record identities.
    pub fn is_historical_selection(&self) -> bool {
        matches!(
            self.origin.publication,
            worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication::HistoricalSelection { .. }
        )
    }
    pub fn authority(&self) -> &worth_proof::RequiredOutputPublicationAuthority {
        &self.authority
    }
    /// Whether both name one canonical Native origin of this runtime: the same
    /// publishing commit, producer, dependency and complete route meaning.
    pub fn same_origin(&self, other: &Self) -> bool {
        self.runtime_instance_id == other.runtime_instance_id
            && self.origin.commit == other.origin.commit
            && self.origin.producer == other.origin.producer
            && self.origin.dependency_identity == other.origin.dependency_identity
            && self.origin.meaning == other.origin.meaning
    }
}

impl crate::transactions::data::outcomes::commit_result::CommitResult {
    pub fn required_output_publication(
        &self,
        installed: &super::RelationalInstalledApplicationPublicationAuthority,
    ) -> Option<RelationalRequiredOutputPublication> {
        let envelope = &self.publication().envelope;
        (self.snapshot.runtime_instance_id() == installed.runtime_instance_id
            && (*envelope.owner_required_output_origin())
                .as_ref()
                .and_then(|origin| origin.issuer.as_ref())
                .is_some_and(|issuer| issuer.same_publication_owner(&installed.authority)))
        .then(|| RelationalRequiredOutputPublication {
            runtime_instance_id: installed.runtime_instance_id,
            evidence: worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence::Canonical(
                envelope.clone(),
            ),
            origin: Arc::clone(
                (*envelope.owner_required_output_origin())
                    .as_ref()
                    .expect("performed single publication"),
            ),
            readmitted_at: None,
            authority: installed
                .authority
                .seal_required_output_publication(envelope.commit.commit_id.0),
        })
    }
    /// Issue each genuine route's own ordinal seal from this performed commit.
    /// Iteration funds only the reached origins under the original request.
    pub fn required_output_publications<'a>(
        &'a self,
        installed: &'a super::RelationalInstalledApplicationPublicationAuthority,
        execution: &'a worth_execution::ExecutionRequest<'_, '_>,
    ) -> impl Iterator<
        Item = Result<
            RelationalRequiredOutputPublication,
            contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
        >,
    > + 'a{
        let envelope = &self.publication().envelope;
        envelope
            .output_origins()
            .enumerate()
            .filter_map(move |(ordinal, origin)| {
                let selected = execution
                    .run_native_with_checkpoint(|context| {
                        // Reaching a foreign origin still consumes traversal work.
                        checkpoint(context, 1).map_err(execution_stop)?;
                        if self.snapshot.runtime_instance_id() != installed.runtime_instance_id
                            || !origin.issuer.as_ref().is_some_and(|issuer| {
                                issuer.same_publication_owner(&installed.authority)
                            })
                        {
                            return Ok(None);
                        }
                        Ok(Some(RelationalRequiredOutputPublication {
                            runtime_instance_id: installed.runtime_instance_id,
                            evidence: worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence::Canonical(
                                envelope.clone(),
                            ),
                            origin: Arc::clone(origin),
                            readmitted_at: None,
                            authority: installed.authority.seal_required_output_publication_at(
                                envelope.commit.commit_id.0,
                                ordinal,
                            ),
                        }))
                    })
                    .map_err(execution_stop)
                    .and_then(|answer| answer.0);
                match selected {
                    Ok(None) => None,
                    Ok(Some(publication)) => Some(Ok(publication)),
                    Err(cause) => Some(Err(cause)),
                }
            })
    }
}

pub fn readmit_required_output_publication(
    inputs: RetirementHistoryInputs<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &super::RelationalInstalledApplicationPublicationAuthority,
    basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    producer: &str,
    dependency: &[u8; 32],
    meaning: &contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    roles: &[contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget],
) -> Result<
    RelationalRequiredOutputPublication,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    if installed
        .source_scope_ownership
        .scope_kind(producer)
        .is_some()
    {
        installed.require_current_ownership_storage(basis, meaning.source_scope)?;
    }
    if installed
        .source_scope_ownership
        .scope_kind(producer)
        .is_some()
        && roles
            .iter()
            .any(|role| role.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
    {
        return current::readmit(
            execution, installed, basis, producer, dependency, meaning, roles,
        );
    }
    if installed.runtime_instance_id != inputs.projection.runtime_identity {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "foreign installed publisher for cold readmission",
            ),
        );
    }
    let selected = worth_relational_branch::internal_port::history::data::codec_retirement_closure::output_correspondence::validate_codec_output_correspondence(
        execution, inputs, basis, producer, dependency, meaning, roles,
    )
    .map_err(cold_stop)?;
    let worth_relational_branch::internal_port::history::data::codec_retirement_closure::output_correspondence::SelectedOutputPublication {
        evidence,
        origin,
        ordinal,
    } = selected;
    if origin.source_scope_owned {
        installed.require_current_ownership_storage(basis, origin.meaning.source_scope)?;
    }
    if worth_relational_branch::internal_port::history::data::codec_retirement_closure::route_selection::readmit_required_output_route(execution, inputs, basis, meaning)
        .map_err(cold_stop)?
        != Some(
            worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition::Published(origin.commit.commit_id),
        )
    {
        return Err(contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("superseded complete family publication is historical association, not a current predecessor"));
    }
    if installed
        .source_scope_ownership
        .scope_kind(&origin.producer)
        .is_some()
        != origin.source_scope_owned
    {
        return Err(
                contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                    contracts::publication::data::publication_error::PublicationError::authored_retirement(
                        contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::CorruptPublication,
                        contracts::transactions::data::primitives::RecordRef::Entity(origin.meaning.source_scope),
                    ),
                ),
            );
    }
    if origin
        .issuer
        .as_ref()
        .is_some_and(|issuer| !issuer.same_publication_owner(&installed.authority))
    {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "foreign installed owner cannot reissue live publication custody",
            ),
        );
    }
    let authority = installed
        .authority
        .seal_required_output_publication_at(origin.commit.commit_id.0, ordinal);
    Ok(RelationalRequiredOutputPublication {
        runtime_instance_id: installed.runtime_instance_id,
        evidence,
        origin,
        authority,
        readmitted_at: Some(basis.clone()),
    })
}

pub(super) fn cold_stop(
    stop: worth_relational_branch::internal_port::history::data::codec_retirement_closure::CodecRetirementClosureStop,
) -> contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError {
    match stop {
        worth_relational_branch::internal_port::history::data::codec_retirement_closure::CodecRetirementClosureStop::Request(cause) => execution_stop(cause),
        worth_relational_branch::internal_port::history::data::codec_retirement_closure::CodecRetirementClosureStop::Payload(detail) => {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(detail)
        }
    }
}

pub fn execution_stop(
    cause: impl Into<contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError {
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(
        contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
            kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
            partition_identity: None,
        },
    )
}

impl crate::transactions::data::outcomes::commit_result::CommitResult {
    /// Descriptive roles of this commit's canonical primary publication. Closure
    /// witnesses used to admit a succession are not members of that publication.
    /// This view grants no publication or succession custody.
    pub fn primary_output_members(
        &self,
    ) -> Option<
        impl ExactSizeIterator<
            Item = (
                &str,
                &str,
                contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture,
                contracts::identity::data::record_ids::EntityId,
            ),
        >,
    >{
        (*self.publication().envelope.owner_required_output_origin())
            .as_ref()
            .map(|origin| {
                origin
                    .members
                    .iter()
                    .map(|m| (m.role.as_str(), m.entity_name.as_str(), m.posture, m.id))
            })
    }
}

pub mod publication_custody_doors_for_test;

#[doc(hidden)]
pub trait RelationalRequiredOutputPublicationFieldsPort: Sized {
    fn origin(&self) -> &Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>;
}

impl RelationalRequiredOutputPublicationFieldsPort for RelationalRequiredOutputPublication {
    fn origin(&self) -> &Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>{
        &self.origin
    }
}

#[doc(hidden)]
pub trait RelationalRequiredOutputPublicationFixturePort {
    fn replace_recorded_custody_for_test(
        &mut self,
        origin: Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>,
        envelope: Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>,
    );
}

impl RelationalRequiredOutputPublicationFixturePort for RelationalRequiredOutputPublication {
    /// Deliberately substitute another recorded artifact at the corruption receiver.
    fn replace_recorded_custody_for_test(
        &mut self,
        origin: Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>,
        envelope: Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>,
    ) {
        self.origin = origin;
        self.evidence = worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence::canonical(envelope);
    }
}
